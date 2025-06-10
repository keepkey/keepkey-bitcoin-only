use crate::{
    cli::{expect_success, parsers::TypedPossibleValuesParser, CliCommand},
    messages::{self, Message},
    transport::ProtocolAdapter,
};
use anyhow::Result;
use clap::{
    ArgAction::{SetFalse, SetTrue},
    Args,
};
use crossterm::event::{Event, KeyCode, KeyEvent};
use std::io::{stdout, Write};

/// Start safe recovery workflow
#[derive(Debug, Clone, Args)]
pub struct RecoveryDevice {
    #[clap(short, long, default_value_t = 12, value_parser = TypedPossibleValuesParser::<u32>::new(["12", "18", "24"]))]
    word_count: u32,
    #[clap(short = 'r', long, action = SetTrue)]
    passphrase_protection: Option<bool>,
    #[clap(short, long, action = SetTrue)]
    pin_protection: Option<bool>,
    #[clap(short = 'g', long)]
    language: Option<String>,
    #[clap(short, long)]
    label: Option<String>,
    /// don't enforce BIP-39 wordlist during the process
    #[clap(short, long, action = SetFalse)]
    no_enforce_wordlist: Option<bool>,
    /// screensaver timeout
    #[clap(short, long)]
    auto_lock_delay_ms: Option<u32>,
    #[clap(short, long)]
    u2f_counter: Option<u32>,
    /// perform dry-run recovery workflow (for safe mnemonic validation)
    #[clap(short, long, action = SetTrue)]
    dry_run: Option<bool>,
}

impl CliCommand for RecoveryDevice {
    fn handle(self, protocol_adapter: &mut dyn ProtocolAdapter) -> Result<()> {
        let mut printed_char_req_msg = false;
        expect_success!(protocol_adapter
            .with_standard_handler()
            .with_mut_handler(&mut |msg| match msg {
                Message::CharacterRequest(messages::CharacterRequest { character_pos, .. }) => {
                    if !printed_char_req_msg {
                        print!(
                            "Enter your mnemonic using the cipher shown on your device screen: "
                        );
                        stdout().flush().unwrap();
                        printed_char_req_msg = true;
                    }
                    Ok(Some((|| -> crossterm::Result<Message> {
                        loop {
                            match crossterm::event::read()? {
                                Event::Key(KeyEvent {
                                    code: KeyCode::Backspace,
                                    ..
                                }) => {
                                    return Ok(messages::CharacterAck {
                                        character: None,
                                        delete: Some(true),
                                        done: None,
                                    }
                                    .into())
                                }
                                Event::Key(KeyEvent {
                                    code: KeyCode::Enter,
                                    ..
                                }) => {
                                    return Ok(messages::CharacterAck {
                                        character: None,
                                        delete: None,
                                        done: Some(true),
                                    }
                                    .into())
                                }
                                Event::Key(KeyEvent {
                                    code: KeyCode::Char(c),
                                    ..
                                }) if c == ' '
                                    || (*character_pos < 4 && c.is_ascii_lowercase()) =>
                                {
                                    return Ok(messages::CharacterAck {
                                        character: Some(c.into()),
                                        delete: None,
                                        done: None,
                                    }
                                    .into())
                                }
                                _ => (),
                            }
                        }
                    })()?))
                }
                _ => {
                    if printed_char_req_msg {
                        println!();
                    }
                    Ok(None)
                }
            },)
            .handle(
                messages::RecoveryDevice {
                    word_count: Some(self.word_count),
                    passphrase_protection: self.passphrase_protection,
                    pin_protection: self.pin_protection,
                    language: self.language,
                    label: self.label,
                    enforce_wordlist: self.no_enforce_wordlist.map(|x| !x),
                    use_character_cipher: None,
                    auto_lock_delay_ms: self.auto_lock_delay_ms,
                    u2f_counter: self.u2f_counter,
                    dry_run: self.dry_run,
                }
                .into(),
            ))?;

        Ok(())
    }
}
