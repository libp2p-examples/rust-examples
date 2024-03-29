// Copyright 2018 Parity Technologies (UK) Ltd.
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

//! copy from Ping example
//! use msg protocol 
//! See ../src/tutorial.rs for a step-by-step guide building the example below.
//!
//! In the first terminal window, run:
//!
//! ```sh
//! cargo run --example 03-msg
//! ```
//!
//! It will print the PeerId and the listening addresses, e.g. `Listening on
//! "/ip4/0.0.0.0/tcp/24915"`
//!
//! In the second terminal window, start a new instance of the example with:
//!
//! ```sh
//! cargo run --example 03-msg  -- /ip4/127.0.0.1/tcp/24915
//! ```
//!
//! The two nodes establish a connection, negotiate the ping protocol
//! and begin pinging each other.

use futures::{prelude::*,select};

use libp2p::swarm::{Swarm, SwarmEvent};
use libp2p::{identity, Multiaddr, PeerId};
use std::error::Error;
use std::str::FromStr;

use async_std::io;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    let transport = libp2p::development_transport(local_key).await?;

    // Create a ping network behaviour.
    //
    // For illustrative purposes, the ping protocol is configured to
    // keep the connection alive, so a continuous sequence of pings
    // can be observed.
    let behaviour = rust_examples::Behaviour::new();

    let mut swarm = Swarm::new(transport, behaviour, local_peer_id);

    // Tell the swarm to listen on all interfaces and a random, OS-assigned
    // port.
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Dial the peer identified by the multi-address given as the second
    // command-line argument, if any.
    if let Some(addr) = std::env::args().nth(1) {
        let remote: Multiaddr = addr.parse()?;
        swarm.dial(remote)?;
        println!("Dialed {}", addr)
    }

    let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();


    loop {

        select! {
 
            line = stdin.select_next_some() => {
                let sline = line.expect("Stdin not to close");
                let vline :Vec<&str> = sline.split("@").collect();
                let P = PeerId::from_str(&"QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN")?;
                let P = PeerId::from_str(vline[0])?;
                println!("peerid:{:?},content:{:?}",P,vline[1]);
                swarm
                .behaviour_mut()
                .send(vline[1].as_bytes())

            },

            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {:?}", address),
                SwarmEvent::Behaviour(event) => {
                    match event {
                        rust_examples::Event{peer,result} =>{
                            match result {
                                rust_examples::MsgContent{data}=>{
                                    println!("recv:from->{:?}\n {:?}",peer,std::str::from_utf8(&data).unwrap());
                                }
                            }
                        }

                    } ;
                },
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    println!("New connect {:?}", peer_id);
                    swarm.behaviour_mut()
                    .insert(&peer_id);

                    let peers = swarm.connected_peers();
                    for p in peers {
                        println!("peer {}",p);
                    }
                },

                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                    println!("disconnect {:?}", peer_id);
                    swarm.behaviour_mut()
                    .remove(&peer_id);
                },

                //default
                _ => {}
            }
        } // select
    }
}
