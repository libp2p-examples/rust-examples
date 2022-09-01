// Copyright 2020 Parity Technologies (UK) Ltd.
// Copyright 2021 Protocol Labs.
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use clap::Parser;
use futures::executor::block_on;
use futures::stream::StreamExt;
use libp2p::core::upgrade;
use libp2p::identify::{Identify, IdentifyConfig, IdentifyEvent};
use libp2p::multiaddr::Protocol;
use libp2p::relay::v2::relay::{self, Relay};
use libp2p::swarm::{Swarm, SwarmEvent};
use libp2p::tcp::TcpTransport;
use std::time::Duration;
// use libp2p::tcp::TcpConfig;
use libp2p::Transport;
use libp2p::{identity, NetworkBehaviour, PeerId};
use libp2p::{noise, Multiaddr};
use libp2p::rendezvous;
use std::error::Error;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::num::NonZeroU32;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let opt = Opt::parse();
    println!("opt: {:?}", opt);

    // Create a static known PeerId based on given secret
    let local_key: identity::Keypair = generate_ed25519(opt.secret_key_seed);
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    let transport = libp2p::development_transport(local_key.clone()).await?;

    let behaviour = Behaviour {
        relay: Relay::new(local_peer_id, Default::default()),
        identify: Identify::new(IdentifyConfig::new(
            "/TODO/0.0.1".to_string(),
            local_key.public(),
        )),
        rendezvous: rendezvous::server::Behaviour::new(rendezvous::server::Config::default()),
    };

    let mut swarm = Swarm::new(transport, behaviour, local_peer_id);

    // Listen on all interfaces
    let listen_addr = Multiaddr::empty()
        .with(match opt.use_ipv6 {
            Some(true) => Protocol::from(Ipv6Addr::UNSPECIFIED),
            _ => Protocol::from(Ipv4Addr::UNSPECIFIED),
        })
        .with(Protocol::Tcp(opt.port));

    swarm.listen_on(listen_addr)?;

    block_on(async {
        loop {
            match swarm.next().await.expect("Infinite Stream.") {
                SwarmEvent::Behaviour(Event::Relay(event)) => {
                    println!("{:?}", event)
                }
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Listening on {:?}", address);
                }
                SwarmEvent::ConnectionEstablished { peer_id, endpoint, num_established, concurrent_dial_errors } => {
                    println!("ConnectionEstablished: peer_id -> {}", peer_id)
                }
                SwarmEvent::ConnectionClosed { peer_id, endpoint, num_established, cause } => {
                    println!("ConnectionClosed: peer_id -> {}", peer_id)
                }
                SwarmEvent::Behaviour(Event::Rendezvous(
                    rendezvous::server::Event::PeerRegistered { peer, registration },
                )) => {
                    println!(
                        "Peer {} registered for namespace '{}'",
                        peer,
                        registration.namespace
                    );
                }
                SwarmEvent::Behaviour(Event::Rendezvous(
                    rendezvous::server::Event::DiscoverServed {
                        enquirer,
                        registrations,
                    },
                )) => {
                    println!(
                        "Served peer {} with {} registrations",
                        enquirer,
                        registrations.len()
                    );
                }
                other => {
                    log::debug!("Unhandled {:?}", other);
                }
            }
        }
    })
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "Event", event_process = false)]
struct Behaviour {
    relay: Relay,
    identify: Identify,
    rendezvous: rendezvous::server::Behaviour,
}

#[derive(Debug)]
enum Event {
    Identify(IdentifyEvent),
    Relay(relay::Event),
    Rendezvous(rendezvous::server::Event),
}

impl From<rendezvous::server::Event> for Event {
    fn from(event: rendezvous::server::Event) -> Self {
        Event::Rendezvous(event)
    }
}

impl From<IdentifyEvent> for Event {
    fn from(e: IdentifyEvent) -> Self {
        Event::Identify(e)
    }
}

impl From<relay::Event> for Event {
    fn from(e: relay::Event) -> Self {
        Event::Relay(e)
    }
}

fn generate_ed25519(secret_key_seed: u8) -> identity::Keypair {
    let mut bytes = [0u8; 32];
    bytes[0] = secret_key_seed;

    let secret_key = identity::ed25519::SecretKey::from_bytes(&mut bytes)
        .expect("this returns `Err` only if the length is wrong; the length is correct; qed");
    identity::Keypair::Ed25519(secret_key.into())
}

#[derive(Debug, Parser)]
#[clap(name = "libp2p relay")]
struct Opt {
    /// Determine if the relay listen on ipv6 or ipv4 loopback address. the default is ipv4
    #[clap(long)]
    use_ipv6: Option<bool>,

    /// Fixed value to generate deterministic peer id
    #[clap(long,default_value_t = 0)]
    secret_key_seed: u8,

    /// The port used to listen on all interfaces
    #[clap(long,default_value_t = 45678)]
    port: u16,
}
