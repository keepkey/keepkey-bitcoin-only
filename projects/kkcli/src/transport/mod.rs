pub mod protocol_adapter;
pub mod usb;
pub mod hid;

pub use protocol_adapter::*;
pub use usb::*;
pub use hid::*;

use crate::messages::{self, Message};
use anyhow::{anyhow, bail, Result};
use core::time::Duration;
use std::io::{stdin, stdout, Write};
use tracing::{info, debug};

pub trait Transport {
    type Error: std::error::Error;
    fn write(&mut self, msg: &[u8], timeout: Duration) -> Result<usize, Self::Error>;
    fn read(&mut self, buf: &mut Vec<u8>, timeout: Duration) -> Result<(), Self::Error>;
    fn reset(&mut self) -> Result<(), Self::Error>;
}

pub fn standard_message_handler(msg: &Message) -> Result<Option<Message>> {
    info!("StandardHandler: Processing message type: {:?}", msg.message_type());
    
    Ok(match msg {
        Message::ButtonRequest(req) => {
            info!("StandardHandler: ButtonRequest received, code: {:?}", req.code);
            eprintln!("Confirm action on device...");
            let ack = messages::ButtonAck::default();
            info!("StandardHandler: Sending ButtonAck");
            Some(ack.into())
        }
        Message::PinMatrixRequest(x) => {
            match x.r#type {
                Some(t) => match messages::PinMatrixRequestType::from_i32(t)
                    .ok_or_else(|| anyhow!("unrecognized PinMatrixRequestType ({})", t))?
                {
                    messages::PinMatrixRequestType::Current => {
                        eprint!("Enter current PIN: ")
                    }
                    messages::PinMatrixRequestType::NewFirst => eprint!("Enter new PIN: "),
                    messages::PinMatrixRequestType::NewSecond => {
                        eprint!("Re-enter new PIN: ")
                    }
                },
                None => bail!("expected PinMatrixRequestType"),
            }
            stdout().flush().unwrap();
            let mut pin = String::new();
            stdin().read_line(&mut pin)?;
            let pin = pin.trim();
            Some(
                messages::PinMatrixAck {
                    pin: pin.to_owned(),
                }
                .into(),
            )
        }
        Message::PassphraseRequest(_) => {
            eprint!("Enter BIP-39 passphrase: ");
            stdout().flush().unwrap();
            let passphrase = rpassword::read_password()?;
            Some(messages::PassphraseAck { passphrase }.into())
        }
        Message::Failure(x) => bail!("Failure: {}", x.message()),
        _ => None,
    })
}

pub trait ProtocolAdapter {
    fn reset(&mut self) -> Result<()>;
    fn send(&mut self, msg: Message) -> Result<()>;
    fn handle(&mut self, msg: Message) -> Result<Message>;
    fn as_mut_dyn(&mut self) -> &mut dyn ProtocolAdapter;
    fn with_handler<'a: 'b, 'b>(
        &'a mut self,
        handler: &'b MessageHandler<'b>,
    ) -> Box<dyn ProtocolAdapter + 'b> {
        Box::from(MessageHandlerStack {
            parent_adapter: self.as_mut_dyn(),
            handler,
        })
    }
    fn with_mut_handler<'a: 'b, 'b>(
        &'a mut self,
        handler: &'b mut MessageHandlerMut<'b>,
    ) -> Box<dyn ProtocolAdapter + 'b> {
        Box::from(MessageHandlerMutStack {
            parent_adapter: self.as_mut_dyn(),
            handler,
        })
    }
    fn with_standard_handler<'a>(&'a mut self) -> Box<dyn ProtocolAdapter + 'a> {
        self.with_handler(&standard_message_handler)
    }
}

pub type MessageHandler<'a> = dyn Fn(&Message) -> Result<Option<Message>> + 'a;
pub type MessageHandlerMut<'a> = dyn FnMut(&Message) -> Result<Option<Message>> + 'a;

pub struct MessageHandlerStack<'a, 'b> {
    parent_adapter: &'a mut dyn ProtocolAdapter,
    handler: &'b MessageHandler<'b>,
}

pub struct MessageHandlerMutStack<'a, 'b> {
    parent_adapter: &'a mut dyn ProtocolAdapter,
    handler: &'b mut MessageHandlerMut<'b>,
}

impl ProtocolAdapter for MessageHandlerStack<'_, '_> {
    fn reset(&mut self) -> Result<()> {
        self.parent_adapter.reset()
    }
    fn send(&mut self, msg: Message) -> Result<()> {
        self.parent_adapter.send(msg)
    }
    fn handle(&mut self, msg: Message) -> Result<Message> {
        info!("MessageHandlerStack::handle: Starting with message type: {:?}", msg.message_type());
        let mut msg = msg;
        loop {
            info!("MessageHandlerStack::handle: Calling parent adapter...");
            let msg_out = self.parent_adapter.handle(msg)?;
            info!("MessageHandlerStack::handle: Parent returned message type: {:?}", msg_out.message_type());
            
            match (self.handler)(&msg_out)? {
                Some(x) => {
                    info!("MessageHandlerStack::handle: Handler returned message type: {:?}, looping...", x.message_type());
                    msg = x;
                }
                None => {
                    info!("MessageHandlerStack::handle: Handler returned None, done");
                    return Ok(msg_out);
                }
            }
        }
    }
    fn as_mut_dyn(&mut self) -> &mut dyn ProtocolAdapter {
        self
    }
}

impl ProtocolAdapter for MessageHandlerMutStack<'_, '_> {
    fn reset(&mut self) -> Result<()> {
        self.parent_adapter.reset()
    }
    fn send(&mut self, msg: Message) -> Result<()> {
        self.parent_adapter.send(msg)
    }
    fn handle(&mut self, msg: Message) -> Result<Message> {
        let mut msg = msg;
        loop {
            let msg_out = self.parent_adapter.handle(msg)?;
            match (self.handler)(&msg_out)? {
                Some(x) => msg = x,
                None => return Ok(msg_out),
            }
        }
    }
    fn as_mut_dyn(&mut self) -> &mut dyn ProtocolAdapter {
        self
    }
}
