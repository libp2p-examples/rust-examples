

//如果这段测试代码不能运行，请参考文档: http://www.libp2p.net.cn/topic/7



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
use futures::future::FutureExt;
use futures::stream::StreamExt;
use libp2p::core::multiaddr::{Multiaddr, Protocol};
use libp2p::core::transport::OrTransport;
use libp2p::core::upgrade;
use libp2p::dcutr;
use libp2p::dns::DnsConfig;
use libp2p::identify::{Identify, IdentifyConfig, IdentifyEvent, IdentifyInfo};
use libp2p::noise;
use libp2p::relay::v2::client::{self, Client};
use libp2p::swarm::{SwarmBuilder, SwarmEvent};
use libp2p::tcp::{GenTcpConfig, TcpTransport};
use libp2p::Transport;
use libp2p::{identity, NetworkBehaviour, PeerId};
use log::info;
use std::convert::TryInto;
use std::error::Error;
use std::net::Ipv4Addr;
use std::str::FromStr;
use async_std::io;
use futures::{prelude::*};
use libp2p::rendezvous;

#[derive(Debug, Parser)]
#[clap(name = "libp2p DCUtR client")]
struct Opts {

    /// The listening address
    #[clap(long)]
    relay_address: Multiaddr,

}

#[derive(Debug, Parser, PartialEq)]
enum Mode {
    Dial,
    Listen,
}

impl FromStr for Mode {
    type Err = String;
    fn from_str(mode: &str) -> Result<Self, Self::Err> {
        match mode {
            "dial" => Ok(Mode::Dial),
            "listen" => Ok(Mode::Listen),
            _ => Err("Expected either 'dial' or 'listen'".to_string()),
        }
    }
}

const NAMESPACE: &str = "rendezvous";


fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let opts = Opts::parse();

    // relay server  使用固定的peer id, 见server服务器的peerid生成代码 参数 secret_key_seed = 0
    let rendezvous_point = "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN"
        .parse()
        .unwrap();

    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    let (relay_transport, client) = Client::new_transport_and_behaviour(local_peer_id);

    let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
        .into_authentic(&local_key)
        .expect("Signing libp2p-noise static DH keypair failed.");

    let transport = OrTransport::new(
        relay_transport,
        block_on(DnsConfig::system(TcpTransport::new(
            GenTcpConfig::default().port_reuse(true),
        )))
        .unwrap(),
    )
    .upgrade(upgrade::Version::V1)
    .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
    .multiplex(libp2p::yamux::YamuxConfig::default())
    .boxed();

    #[derive(NetworkBehaviour)]
    #[behaviour(out_event = "Event", event_process = false)]
    struct Behaviour {
        relay_client: Client,
        identify: Identify,
        dcutr: dcutr::behaviour::Behaviour,
        sendmsg:rust_examples::Behaviour,
        rendezvous:rendezvous::client::Behaviour,

    
        #[behaviour(ignore)]
        #[allow(dead_code)]

        has_registered:bool,
    }

    #[derive(Debug)]
    enum Event {
        Identify(IdentifyEvent),
        Relay(client::Event),
        Dcutr(dcutr::behaviour::Event),
        Send(rust_examples::Event),
        Rendezvous(rendezvous::client::Event),
    }


    impl From<IdentifyEvent> for Event {
        fn from(e: IdentifyEvent) -> Self {
            Event::Identify(e)
        }
    }
    impl From<rendezvous::client::Event> for Event {
        fn from(e: rendezvous::client::Event) -> Self {
            Event::Rendezvous(e)
        }
    }

    impl From<client::Event> for Event {
        fn from(e: client::Event) -> Self {
            Event::Relay(e)
        }
    }

    impl From<dcutr::behaviour::Event> for Event {
        fn from(e: dcutr::behaviour::Event) -> Self {
            Event::Dcutr(e)
        }
    }

    impl From<rust_examples::Event> for Event {
        fn from(e: rust_examples::Event) -> Self {
            Event::Send(e)
        }
    }

    let behaviour = Behaviour {
        relay_client: client,
        identify: Identify::new(IdentifyConfig::new(
            "/TODO/0.0.1".to_string(),
            local_key.public(),
        )),
        dcutr: dcutr::behaviour::Behaviour::new(),
        sendmsg: rust_examples::Behaviour::new(),
        rendezvous: rendezvous::client::Behaviour::new(local_key.clone()),

        has_registered: false,
    };

    let mut cookie = None;

    let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();

    let mut swarm = SwarmBuilder::new(transport, behaviour, local_peer_id)
        .dial_concurrency_factor(10_u8.try_into().unwrap())
        .build();

    swarm
        .listen_on(
            Multiaddr::empty()
                .with("0.0.0.0".parse::<Ipv4Addr>().unwrap().into())
                .with(Protocol::Tcp(0)),
        )
        .unwrap();

    // Wait to listen on all interfaces.
    block_on(async {
        let mut delay = futures_timer::Delay::new(std::time::Duration::from_millis(100)).fuse();
        loop {
            futures::select! {
                event = swarm.next() => {
                    match event.unwrap() {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            println!("Listening on {:?}", address);
                        }
                        event => panic!("{:?}", event),
                    }
                }

                _ = delay => {
                    // Likely listening on all interfaces now, thus continuing by breaking the loop.
                    break;
                }
            }
        }
    });

    // Connect to the relay server. Not for the reservation or relayed connection, but to (a) learn
    // our local public address and (b) enable a freshly started relay to learn its public address.
    swarm.dial(opts.relay_address.clone()).unwrap();

    block_on(async {
        let mut learned_observed_addr = false;
        let mut told_relay_observed_addr = false;

        loop {
            match swarm.next().await.unwrap() {
                SwarmEvent::NewListenAddr { .. } => {}
                SwarmEvent::Dialing { .. } => {}

                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    info!("{}",peer_id);
                },


                SwarmEvent::Behaviour(Event::Identify(IdentifyEvent::Sent { .. })) => {
                    info!("Told relay its public address.");
                    told_relay_observed_addr = true;
                }
                SwarmEvent::Behaviour(Event::Identify(IdentifyEvent::Received {
                    info: IdentifyInfo { observed_addr, .. },
                    ..
                })) => {
                    println!("Relay told us our public address: {:?}", observed_addr);
                    learned_observed_addr = true;

                    swarm.behaviour_mut().rendezvous.register(
                        rendezvous::Namespace::from_static("rendezvous"),
                        rendezvous_point,
                        None,
                        );


                }
                event => info!("{:?}", event),
            }

            if learned_observed_addr && told_relay_observed_addr {
                break;
            }
        }
    });
    /*
    match opts.mode {
        Mode::Dial => {
            swarm
                .dial(
                    opts.relay_address
                        .with(Protocol::P2pCircuit)
                        .with(Protocol::P2p(opts.remote_peer_id.unwrap().into())),
                )
                .unwrap();
        }
        Mode::Listen => {
            swarm
                .listen_on(opts.relay_address.with(Protocol::P2pCircuit))
                .unwrap();
        }
    }*/


    swarm
       .listen_on(opts.relay_address.clone().with(Protocol::P2pCircuit))
       .unwrap();

    block_on(async {
        loop {
          futures::select! {

            line = stdin.select_next_some() => swarm
                .behaviour_mut()
                .sendmsg
                .send(line.expect("Stdin not to close").as_bytes()),



            event = swarm.select_next_some() => match event{
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Listening on {:?}", address);
                }
                SwarmEvent::Behaviour(Event::Relay(client::Event::ReservationReqAccepted {
                    ..
                })) => {
                    info!("Relay accepted our reservation request.");
                }
                SwarmEvent::Behaviour(Event::Relay(event)) => {
                    info!("{:?}", event)
                }
                SwarmEvent::Behaviour(Event::Dcutr(event)) => {
                    info!("{:?}", event)
                }
                SwarmEvent::Behaviour(Event::Send(event)) => {
                    println!("{:?}", event)
                }
                SwarmEvent::Behaviour(Event::Identify(event)) => {
                    info!("{:?}", event)
                }
                SwarmEvent::ConnectionEstablished {
                    peer_id, endpoint, ..
                } => {
                    println!("Established connection to {:?} via {:?}", peer_id, endpoint);
                    swarm.behaviour_mut()
                        .sendmsg
                        .insert(&peer_id);

                    let peers = swarm.connected_peers();
                    for p in peers {
                        println!("peer {}",p);
                    }

                }
                SwarmEvent::Behaviour(Event::Rendezvous(rendezvous::client::Event::Registered {
                    namespace,
                    ttl,
                    rendezvous_node,
                })) => {
                    println!(
                        "Registered for namespace '{}' at rendezvous point {} for the next {} seconds",
                        namespace,
                        rendezvous_node,
                        ttl
                    );
                    swarm.behaviour_mut().has_registered = true;
                            
                    let behaviour = swarm.behaviour_mut();
                
                    behaviour.rendezvous.discover(
                        Some(rendezvous::Namespace::new(NAMESPACE.to_string()).unwrap()),
                        None,
                        None,
                        rendezvous_point
                    );

                }
                SwarmEvent::Behaviour(Event::Rendezvous(rendezvous::client::Event::Discovered {
                        registrations,
                        cookie: new_cookie,
                        ..
                    })) => {
                        cookie.replace(new_cookie);

                        for registration in registrations {
                            for address in registration.record.addresses() {
                                let peer = registration.record.peer_id();
                                println!("Discovered peer {} at {}", peer, address);

                                let p2p_suffix = Protocol::P2p(*peer.as_ref());
                                let address_with_p2p =
                                    if !address.ends_with(&Multiaddr::empty().with(p2p_suffix.clone())) {
                                        address.clone().with(p2p_suffix)
                                    } else {
                                        address.clone()
                                    };
                                
                                //swarm.dial(address_with_p2p).unwrap()
                                swarm
                                .dial(
                                    opts.relay_address.clone()
                                    .with(Protocol::P2pCircuit)
                                    .with(Protocol::P2p(peer.into())),
                                     )
                                    .unwrap();
                                println!("Dial {}",opts.relay_address.clone()
                                    .with(Protocol::P2pCircuit)
                                    .with(Protocol::P2p(peer.into())) );
                            }
                        }
                    }
 

                SwarmEvent::ConnectionClosed { peer_id,endpoint ,.. } => {
                    println!("disconnect {:?} by {:?}", peer_id,endpoint);
                    /*swarm.behaviour_mut()
                    .sendmsg
                    .remove(&peer_id);*/
                },


                SwarmEvent::OutgoingConnectionError { peer_id, error } => {
                    info!("Outgoing connection error to {:?}: {:?}", peer_id, error);
                }
                _ => {}
            }
        } //select 
        }//loop
    })
}

