use super::{ProtocolAdapter, Transport};
use crate::messages::Message;
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use std::sync::RwLock;
use log::{info, debug};

lazy_static! {
    pub static ref VERBOSE: RwLock<bool> = RwLock::new(false);
}

impl<T, E> ProtocolAdapter for T
where
    T: Transport<Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    fn reset(&mut self) -> Result<()> {
        Ok(<T as Transport>::reset(self)?)
    }

    fn send(&mut self, msg: Message) -> Result<()> {
        info!("ProtocolAdapter::send: Sending message type: {:?}", msg.message_type());
        
        if *VERBOSE.read().unwrap() {
            println!("-> {:?}", msg);
        }
        let mut out_buf = Vec::<u8>::with_capacity(msg.encoded_len());
        msg.encode(&mut out_buf)?;
        
        debug!("ProtocolAdapter::send: Encoded message size: {} bytes", out_buf.len());
        
        self.write(&out_buf, msg.write_timeout())?;

        Ok(())
    }

    fn as_mut_dyn(&mut self) -> &mut dyn ProtocolAdapter {
        self
    }

    fn handle(&mut self, msg: Message) -> Result<Message> {
        info!("ProtocolAdapter::handle: Processing message type: {:?}", msg.message_type());
        
        let read_timeout = msg.read_timeout();
        self.send(msg)?;

        info!("ProtocolAdapter::handle: Waiting for response (timeout: {:?})...", read_timeout);
        let mut in_buf = Vec::<u8>::new();
        self.read(&mut in_buf, read_timeout)?;
        
        info!("ProtocolAdapter::handle: Received {} bytes response", in_buf.len());

        let out = Message::decode(&mut in_buf.as_slice()).map_err(|x| anyhow!(x))?;
        info!("ProtocolAdapter::handle: Decoded response type: {:?}", out.message_type());
        
        if *VERBOSE.read().unwrap() {
            println!("<- {:?}", out);
        }
        Ok(out)
    }
}
