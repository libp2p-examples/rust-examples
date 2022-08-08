// Copyright 2019 Parity Technologies (UK) Ltd.
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

use crate::protocol;
use libp2p::swarm::{
    ConnectionHandler, ConnectionHandlerEvent, ConnectionHandlerUpgrErr, KeepAlive,
    SubstreamProtocol,
};
use std::collections::VecDeque;
use std::{
    task::{Context, Poll},
};


/// The successful result of processing an inbound or outbound ping.
#[derive(Debug)]
pub enum Success {
    OK,
}

/// Protocol handler that handles pinging the remote at a regular period
/// and answering ping queries.
///
/// If the remote doesn't respond, produces an error that closes the connection.
pub struct Handler {

    /// Outbound Inbound events
    queued_events: VecDeque<
        ConnectionHandlerEvent<
            <Self as ConnectionHandler>::OutboundProtocol,
            <Self as ConnectionHandler>::OutboundOpenInfo,
            <Self as ConnectionHandler>::OutEvent,
            <Self as ConnectionHandler>::Error,
        >,
    >,

}



impl Handler {
    /// Builds a new `PingHandler` with the given configuration.
    pub fn new() -> Self {
        Handler {
            queued_events: Default::default(),
            
        }
    }
}

impl ConnectionHandler for Handler {
    type InEvent = protocol::MsgContent;
    type OutEvent = protocol::MsgContent;
    type Error = std::io::Error;
    type InboundProtocol = protocol::MsgContent;
    type OutboundProtocol = protocol::MsgContent;
    type OutboundOpenInfo = ();
    type InboundOpenInfo = ();

    fn listen_protocol(&self) -> SubstreamProtocol<protocol::MsgContent, ()> {
        SubstreamProtocol::new(protocol::MsgContent{data:Default::default()}, ())
    }

    //protocol::InboundUpgrade::Output  
    fn inject_fully_negotiated_inbound(&mut self, output: Vec<u8>, (): ()) {
        //println!("inject_fully_negotiated_inbound ");
        
        self.queued_events.push_back(ConnectionHandlerEvent::Custom(
                protocol::MsgContent{data:output}));
    }

    fn inject_fully_negotiated_outbound(&mut self, _output: protocol::Success, (): ()) {

        //println!("inject_fully_negotiated_outbound");
        
    }

    fn inject_event(&mut self, msg:protocol::MsgContent ) {
        //println!("handler inject event ");
        self.queued_events.push_back(
                ConnectionHandlerEvent::OutboundSubstreamRequest {
                        protocol: SubstreamProtocol::new(
                            msg,
                            (),
                        )}
            );
    }

    fn inject_dial_upgrade_error(&mut self, _info: (), _error: ConnectionHandlerUpgrErr<std::io::Error>) {
       

    }

    fn connection_keep_alive(&self) -> KeepAlive { KeepAlive::Yes }

    fn poll(
        &mut self,
        _cx: &mut Context<'_>,
    ) -> Poll<ConnectionHandlerEvent<protocol::MsgContent, (), protocol::MsgContent, Self::Error>> {


        if let Some(msg) = self.queued_events.pop_back(){
            return Poll::Ready(msg);
        }        

        Poll::Pending
    }
}


