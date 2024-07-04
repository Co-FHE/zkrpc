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
}

pub async fn start_bootstrap_server(cfg: BootstrapConfig) -> anyhow::Result<()> {
    let key = match cfg.fixed_seed {
        Some(seed) => libp2p::identity::Keypair::ed25519_from_bytes(seed)?,
        None => libp2p::identity::Keypair::generate_ed25519(),
    };

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

            other => {
                tracing::debug!("Unhandled {:?}", other);
            }
        }
    }

    Ok(())
}
