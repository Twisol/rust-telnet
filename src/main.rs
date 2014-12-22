#![feature(slicing_syntax, globs, unboxed_closures)]
//#![allow(dead_code, unused_parens)]
use parser::{TelnetTokenizer, TelnetToken};

mod parser;

const SE: u8 = 240;
const SB: u8 = 250;
const WILL: u8 = 251;
const WONT: u8 = 252;
const DO: u8 = 253;
const DONT: u8 = 254;

#[deriving(Copy)]
pub struct QState {
  pub status: [u8, ..256]
}

impl QState {
  pub fn new() -> QState {
    QState{status: [0, ..256]}
  }

  pub fn is_channel_enabled(&self, channel: u8) -> bool {
    channel == 32
  }
}

#[deriving(Copy)]
pub struct TelnetContext {
  //| Implemented by WILL, WONT, DO, DONT
  pub qstate: QState,
  //| Implemented by SB, SE
  pub active_channel: Option<u8>,
}

impl TelnetContext {
  pub fn new() -> TelnetContext {
    TelnetContext {
      qstate: QState::new(),
      active_channel: None,
    }
  }
}

fn main() {
  let stream = [b"\xFF\xFA\x20hello, w\xFF\xFForld!\xFF", b"\xF0"];

  //| Tokenizes a telnet stream.
  let mut tokenizer = TelnetTokenizer::new();
  let mut context = TelnetContext::new();

  for &data in stream.iter() {
    for token in tokenizer.tokenize(data) {
      match token {
        TelnetToken::Text(text) => {
          match context.active_channel {
            Some(ch) => {
              if context.qstate.is_channel_enabled(ch) {
                println!("[{}]: {}", ch, text);
              }
            }
            None => {
              println!("[M]: {}", text);
            }
          }
        }
        TelnetToken::Command(command) => {
          match command {
            SE => {
              context.active_channel = None;
            }

            _ => {}
          }
          println!("IAC {}", command);
        }
        TelnetToken::Negotiation{command, channel} => {
          match command {
            WILL => {}
            WONT => {}
            DO => {}
            DONT => {}

            SB => {
              context.active_channel = Some(channel);
            }

            _ => {}
          }
          println!("IAC {} {}", command, channel);
        }
      }
    }
  }
}
