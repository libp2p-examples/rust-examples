//
// 第一个例子生成一个 peer，同时他也是一个 public keys
//
// cargo build --example 02-peer
// ./target/debug/examples/02-peer  /ip4/127.0.0.1/tcp/57643
// 57643 为 01-peer 端口

use libp2p::{
    identity,
    PeerId,
    Multiaddr,
};

use futures::prelude::*;

use libp2p::swarm::{Swarm, SwarmEvent};
use libp2p::identify::{    Identify, IdentifyConfig, IdentifyEvent};

use std::error::Error;


#[async_std::main]
async fn main() -> Result<(),Box<dyn Error>>{

    // Create a random PeerId.
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());
    println!("Local peer id: {:?}", peer_id);


    // 生成一个 transport
    let transport = libp2p::development_transport(id_keys.clone()).await?;


    // Create a identify network behaviour.
    let behaviour = Identify::new(IdentifyConfig::new(
        "/ipfs/id/1.0.0".to_string(),
        id_keys.public(),
    )); 

    let mut swarm = Swarm::new(transport, behaviour, peer_id);

    if let Some(addr) = std::env::args().nth(1) {
        let remote: Multiaddr = addr.parse()?;
        swarm.dial(remote)?;
        println!("Dialed {}", addr)
    }


    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {:?}", address),
            SwarmEvent::Behaviour(event) => println!("{:?}", event),
            SwarmEvent::Behaviour(IdentifyEvent::Sent { peer_id, .. }) => {
                println!("Sent identify info to {:?}", peer_id)
            }
            // Prints out the info received via the identify event
            SwarmEvent::Behaviour(IdentifyEvent::Received { info, .. }) => {
                println!("Received {:?}", info)
            }
            _ => {}
        }
    }




    return Ok(());
}
