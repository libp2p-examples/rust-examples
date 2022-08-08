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

use futures::prelude::*;
use libp2p::core::{upgrade,InboundUpgrade, OutboundUpgrade, UpgradeInfo};
use libp2p::swarm::NegotiatedSubstream;
use std::{io, iter };
use futures::future::BoxFuture;

/// read and write msg demo

pub enum Success{
    OK,
}

#[derive(Default, Debug, Clone)]
pub struct MsgContent{
    pub data: Vec<u8>,
}


impl UpgradeInfo for MsgContent {
    type Info = &'static [u8];
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        iter::once(b"/p2p/msg/1.0.0")
    }

}

impl InboundUpgrade<NegotiatedSubstream> for MsgContent {
    type Output = Vec<u8>;
    type Error = std::io::Error;
    type Future = BoxFuture<'static, Result<Self::Output, Self::Error>>;

    fn upgrade_inbound(self, socket:NegotiatedSubstream, _: Self::Info) -> Self::Future {
        //println!("upgrade_inbound");
        async move {
            let packet = recv(socket).await?;
            Ok(packet)
        }.boxed()

    }
}

impl OutboundUpgrade<NegotiatedSubstream> for MsgContent {
    type Output = Success;
    type Error = std::io::Error;
    type Future = BoxFuture<'static, Result<Self::Output, Self::Error>>;
    fn upgrade_outbound(self, socket: NegotiatedSubstream, _: Self::Info) -> Self::Future {
        //println!("upgrade_outbound {:?}",self.data);
        async move { 
            send(socket,self.data).await?;
            Ok(Success::OK)
        }.boxed()

    }
}

pub async fn recv<S>(mut socket:S) -> io::Result<Vec<u8>>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    
    let packet = upgrade::read_length_prefixed(&mut socket, 2048).await?;
    //println!("{:?}",std::str::from_utf8(&packet).unwrap());
    
    Ok(packet)
}
pub async fn send<S>(mut socket:S,data: Vec<u8>) -> io::Result<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    
    upgrade::write_length_prefixed(&mut socket, data).await?;
    socket.close().await?;
    Ok(socket)
}
