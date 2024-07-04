use config::BootstrapConfig;
use libp2p::{
    futures::StreamExt,
    gossipsub, identify,
    multiaddr::Protocol,
    noise, rendezvous,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr,
};
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    time::Duration,
};
use tokio::time;
#[derive(NetworkBehaviour)]
struct RegisterBehaviour {
    identify: identify::Behaviour,
    rendezvous: rendezvous::client::Behaviour,
}
pub(crate) async fn bootstrap_identify_registry(cfg: &BootstrapConfig) {
    let rendezvous_point_address = format!("/ip4/127.0.0.1/tcp/{}", cfg.bootstrap_port)
        .parse::<Multiaddr>()
        .unwrap();
    let key = libp2p::identity::Keypair::ed25519_from_bytes(cfg.fixed_seed.unwrap()).unwrap();
    let rendezvous_point = key.public().to_peer_id();
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .unwrap()
        .with_behaviour(|key| RegisterBehaviour {
            identify: identify::Behaviour::new(identify::Config::new(
                "remote_node/1.0.0".to_string(),
                key.public(),
            )),
            rendezvous: rendezvous::client::Behaviour::new(key.clone()),
        })
        .unwrap()
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(5)))
        .build();

    let mut cookie = None;
    let _ = swarm.listen_on("/ip4/0.0.0.0/tcp/15867".parse().unwrap());
    loop {
        if let Err(err) = swarm.dial(rendezvous_point_address.clone()) {
            tracing::error!("Failed to dial rendezvous point: {:?}", err);
            time::sleep(Duration::from_secs(5)).await;
            continue;
        }

        while let Some(event) = swarm.next().await {
            match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    tracing::info!("Listening on {}", address);
                }
                SwarmEvent::ConnectionClosed {
                    peer_id,
                    cause: Some(error),
                    ..
                } if peer_id == rendezvous_point => {
                    tracing::info!("Lost connection to rendezvous point {}", error);
                    break;
                }
                // once `/identify` did its job, we know our external address and can register
                SwarmEvent::Behaviour(RegisterBehaviourEvent::Identify(
                    identify::Event::Received { peer_id, info },
                )) => {
                    if peer_id != rendezvous_point {
                        continue;
                    }
                    tracing::info!("Identified as {}, info {:?}", peer_id, info);
                    let components = info.observed_addr.iter().collect::<Vec<_>>();
                    if components.len() < 2 {
                        tracing::error!("Expected at least 2 components in observed address");
                        continue;
                    }
                    let observed_addr = Multiaddr::empty()
                        .with(components[0].clone())
                        .with(Protocol::Tcp(15867));
                    swarm.add_external_address(observed_addr);
                    if let Err(error) = swarm.behaviour_mut().rendezvous.register(
                        rendezvous::Namespace::from_static("node_registry"),
                        rendezvous_point,
                        None,
                    ) {
                        tracing::error!("Failed to register: {error}");
                        break;
                    }
                }
                SwarmEvent::Behaviour(RegisterBehaviourEvent::Rendezvous(
                    rendezvous::client::Event::Discovered {
                        registrations,
                        cookie: new_cookie,
                        ..
                    },
                )) => {
                    cookie.replace(new_cookie);
                    for registration in registrations {
                        for address in registration.record.addresses() {
                            let peer = registration.record.peer_id();
                            tracing::info!(%peer, %address, "Discovered peer");
                            if peer == rendezvous_point || peer == swarm.local_peer_id().clone() {
                                continue;
                            }
                            let p2p_suffix = Protocol::P2p(peer);
                            let address_with_p2p = if !address
                                .ends_with(&Multiaddr::empty().with(p2p_suffix.clone()))
                            {
                                address.clone().with(p2p_suffix)
                            } else {
                                address.clone()
                            };

                            swarm.dial(address_with_p2p).unwrap();
                        }
                    }
                }
                SwarmEvent::Behaviour(RegisterBehaviourEvent::Rendezvous(
                    rendezvous::client::Event::Registered {
                        namespace,
                        ttl,
                        rendezvous_node,
                    },
                )) => {
                    tracing::info!(
                            "Registered for namespace '{}' at rendezvous point {} for the next {} seconds",
                            namespace,
                            rendezvous_node,
                            ttl
                        );

                    swarm.behaviour_mut().rendezvous.discover(
                        Some(rendezvous::Namespace::new("node_registry".to_string()).unwrap()),
                        None,
                        None,
                        rendezvous_point,
                    );
                }
                SwarmEvent::Behaviour(RegisterBehaviourEvent::Rendezvous(
                    rendezvous::client::Event::RegisterFailed {
                        rendezvous_node,
                        namespace,
                        error,
                    },
                )) => {
                    tracing::error!(
                        "Failed to register: rendezvous_node={}, namespace={}, error_code={:?}",
                        rendezvous_node,
                        namespace,
                        error
                    );
                    break;
                }
                // SwarmEvent::Behaviour(MyBehaviourEvent::Ping(ping::Behaviour::Event {
                //     peer,
                //     result: Ok(rtt),
                //     ..
                // })) if peer != rendezvous_point => {
                //     tracing::info!("Ping to {} is {}ms", peer, rtt.as_millis())
                // }
                SwarmEvent::OutgoingConnectionError {
                    connection_id,
                    peer_id,
                    error,
                } => {
                    tracing::error!("Failed to dial peer {:?}, error: {:?}", peer_id, error);

                    break;
                }
                other => {
                    tracing::debug!("Unhandled {:?}", other);
                }
            }
        }
        time::sleep(Duration::from_secs(5)).await;
    }
}

// We create a custom network behaviour that combines Gossipsub and Mdns.
#[derive(NetworkBehaviour)]
struct MyBehaviour {
    gossipsub: gossipsub::Behaviour,
}
//cargo test --package p2p --lib -- bootstrap::client::aaa --exact --show-output
#[tokio::test]
pub async fn test_gossip_client() -> Result<(), anyhow::Error> {
    let _guard = logger::init_logger_for_test!();
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            // To content-address message, we can take the hash of message and use it as an ID.
            let message_id_fn = |message: &gossipsub::Message| {
                let mut s = DefaultHasher::new();
                message.data.hash(&mut s);
                gossipsub::MessageId::from(s.finish().to_string())
            };

            // Set a custom gossipsub configuration
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
                .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
                .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
                .build()
                .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?; // Temporary hack because `build` does not return a proper `std::error::Error`.

            // build a gossipsub network behaviour
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )?;

            Ok(MyBehaviour { gossipsub })
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    // Create a Gossipsub topic
    let topic = gossipsub::IdentTopic::new("test-net");
    // subscribes to our topic
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    // Read full lines from stdin
    let mut stdin = io::BufReader::lines(io::BufReader::new(io::stdin()));

    // Listen on all interfaces and whatever port the OS assigns
    // swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/444".parse()?)?;
    swarm.behaviour_mut().gossipsub.add_explicit_peer(
        &"12D3KooWMcHQeidWjExfCQ58f8NJ7axnvH2bhpR8SuWsY1NY5T7o"
            .parse()
            .unwrap(),
    );
    swarm
        .dial("/ip4/127.0.0.1/tcp/123".parse::<Multiaddr>().unwrap())
        .unwrap();

    use tokio::{io, io::AsyncBufReadExt, select};
    println!("Enter messages via STDIN and they will be sent to connected peers using Gossipsub");
    // Kick it off
    loop {
        select! {
            Ok(Some(line)) = stdin.next_line() => {
                if let Err(e) = swarm
                    .behaviour_mut().gossipsub
                    .publish(topic.clone(), line.as_bytes()) {
                    println!("Publish error: {e:?}");
                }
            }
            event = swarm.select_next_some() => match event {

                SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: id,
                    message,
                })) => println!(
                        "Got message: '{}' with id: {id} from peer: {peer_id}",
                        String::from_utf8_lossy(&message.data),
                    ),
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Local node is listening on {address}");
                }
                _ => {}
            }
        }
    }
    return Ok(());
}
