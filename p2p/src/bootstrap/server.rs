use std::{
    hash::{DefaultHasher, Hash, Hasher},
    io,
    time::Duration,
};

use config::BootstrapConfig;
use libp2p::{
    autonat,
    futures::StreamExt,
    gossipsub, identify, noise, ping, rendezvous,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, SwarmBuilder,
};
use tracing::info;
#[derive(NetworkBehaviour)]
struct BootstrapBehaviour {
    identify: identify::Behaviour,
    rendezvous: rendezvous::server::Behaviour,
    gossipsub: gossipsub::Behaviour,
}

pub async fn start_bootstrap_server(cfg: BootstrapConfig) -> anyhow::Result<()> {
    let key = match cfg.fixed_seed {
        Some(seed) => libp2p::identity::Keypair::ed25519_from_bytes(seed)?,
        None => libp2p::identity::Keypair::generate_ed25519(),
    };
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
        .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg)); // Temporary hack because `build` does not return a proper `std::error::Error`.

    // build a gossipsub network behaviour
    let gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(key.clone()),
        gossipsub_config.unwrap(),
    )
    .unwrap();

    let mut swarm = SwarmBuilder::with_existing_identity(key.clone())
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| BootstrapBehaviour {
            identify: identify::Behaviour::new(identify::Config::new(
                "bootstrap/0.0.1".to_string(),
                key.public(),
            )),
            rendezvous: rendezvous::server::Behaviour::new(rendezvous::server::Config::default()),
            gossipsub,
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(5)))
        .build();

    let _ = swarm.listen_on(
        format!("/ip4/0.0.0.0/tcp/{}", cfg.bootstrap_port)
            .parse()
            .unwrap(),
    );
    info!(
        "Bootstrap Node listening on port {}, peer id: {}",
        cfg.bootstrap_port,
        swarm.local_peer_id()
    );
    let topic = gossipsub::IdentTopic::new("forward");
    swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();
    while let Some(event) = swarm.next().await {
        match event {
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                tracing::info!("Connected to {}", peer_id);
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                tracing::info!("Disconnected from {}", peer_id);
            }
            SwarmEvent::Behaviour(BootstrapBehaviourEvent::Rendezvous(
                rendezvous::server::Event::PeerRegistered { peer, registration },
            )) => {
                tracing::info!(
                    "Peer {} registered for namespace '{}'",
                    peer,
                    registration.namespace
                );
            }
            SwarmEvent::Behaviour(BootstrapBehaviourEvent::Rendezvous(
                rendezvous::server::Event::DiscoverServed {
                    enquirer,
                    registrations,
                },
            )) => {
                tracing::info!(
                    "Served peer {} with {} registrations",
                    enquirer,
                    registrations.len()
                );
            }

            SwarmEvent::Behaviour(BootstrapBehaviourEvent::Gossipsub(
                gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: id,
                    message,
                },
            )) => {
                info!(
                    "Got message: '{}' with id: {id} from peer: {peer_id}",
                    String::from_utf8_lossy(&message.data),
                );
                if let Err(e) = swarm
                    .behaviour_mut()
                    .gossipsub
                    .publish(topic.clone(), message.data)
                {
                    println!("Publish error: {e:?}");
                }
            }
            other => {
                tracing::debug!("Unhandled {:?}", other);
            }
        }
    }

    Ok(())
}
