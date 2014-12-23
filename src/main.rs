#![feature(slicing_syntax, globs, unboxed_closures, macro_rules)]
//#![allow(dead_code, unused_parens)]
use parser::{TelnetTokenizer, TelnetToken};
use std::collections::{VecMap};
use std::cell::{RefCell};

mod parser;

const SE: u8 = 240;
const SB: u8 = 250;
const WILL: u8 = 251;
const WONT: u8 = 252;
const DO: u8 = 253;
const DONT: u8 = 254;

#[deriving(Show, Copy, PartialEq)]
pub enum QStateUni {
  Disabled,
  AwaitEnable,
  Enabled,
  AwaitDisable,
}

impl QStateUni {
  pub fn is_enabled(self) -> bool {
    self == QStateUni::Enabled || self == QStateUni::AwaitDisable
  }

  pub fn is_disabled(self) -> bool {
    !self.is_enabled()
  }
}

#[deriving(Show, Copy)]
pub struct QState {
  pub local: QStateUni,
  pub remote: QStateUni,
}

impl QState {
  pub fn new() -> QState {
    QState{local: QStateUni::Disabled, remote: QStateUni::Disabled}
  }

  pub fn is_active(&self) -> bool {
    self.local.is_enabled() || self.remote.is_enabled()
  }
}

pub struct TelnetCore<'a> {
  pub active_channel: Option<u8>,
  pub qstate: QState,
  pub channels: VecMap<(QState, &'a (ChannelEndpoint + 'a))>,
  pub commands: VecMap<&'a (CommandEndpoint + 'a)>,
}

impl<'a> TelnetCore<'a> {
  pub fn new() -> TelnetCore<'a> {
    TelnetCore {
      qstate: QState::new(),
      active_channel: None,
      channels: VecMap::new(),
      commands: VecMap::new(),
    }
  }
}

fn main() {
  let stream = [b"abc", b"def\xFF\xFA\x20hello, w\xFF\xFForld!\xFF", b"\xF0"];

  let mut tokenizer = TelnetTokenizer::new();
  let mut context = TelnetCore::new();
  let foo = Foo(RefCell::new(Bar(42)));
  context.channels.insert(32, (QState::new(), &foo));
  context.commands.insert(240, &foo);

  for &data in stream.iter() {
    for token in tokenizer.tokenize(data) {
      context.dispatch(token);
    }
  }
}

pub trait ChannelEndpoint {
  fn on_data<'a>(&self, _: Option<u8>, _: &'a [u8]) {}
  fn on_enable(&self, _: Option<u8>) {}
  fn on_disable(&self, _: Option<u8>) {}
  fn on_focus(&self, _: Option<u8>) {}
  fn on_blur(&self, _: Option<u8>) {}
}

pub trait CommandEndpoint {
  fn on_command(&self, _: Option<u8>, _: u8) {}
}

pub trait NegotiableChannel: ChannelEndpoint {
  fn should_enable(&self) -> bool { false }
}

struct DefaultEndpoint;
impl ChannelEndpoint for DefaultEndpoint {
  fn on_data<'a>(&self, channel: Option<u8>, data: &'a [u8]) {
    match channel {
      None     => println!("[{}]: {}", 'M', data),
      Some(ch) => println!("[{}]: {}", ch.to_string(), data),
    }
  }
}
impl CommandEndpoint for DefaultEndpoint {
  fn on_command(&self, channel: Option<u8>, cmd: u8) {
    match channel {
      None     => println!("IAC {}", cmd),
      Some(ch) => println!("IAC {} {}", cmd, ch),
    }
  }
}

struct Bar(u8);
struct Foo<'a>(RefCell<Bar>);
impl<'a> ChannelEndpoint for Foo<'a> {
  fn on_data<'b>(&self, _channel: Option<u8>, text: &'b [u8]) {
    let mut bar = (self.0).borrow_mut();
    bar.0 += 1;
    println!("[FOO]: {} {}", bar.0, text);
  }
}

impl<'a> CommandEndpoint for Foo<'a> {
  fn on_command(&self, _channel: Option<u8>, _command: u8) {
    let mut bar = (self.0).borrow_mut();
    bar.0 += 1;

    println!("TEST TEST {}", bar.0);
  }
}

impl<'a> TelnetCore<'a> {
  fn dispatch(&mut self, token: TelnetToken) {
    match token {
      TelnetToken::Text(text) => {
        let default = DefaultEndpoint;
        let channel = match self.active_channel {
          Some(ch) => {
            match self.channels.get_mut(&(ch as uint)) {
              Some(&(_, ref channel)) => { &**channel }
              None => { &default as &ChannelEndpoint }
            }
          }
          None => {
            &default as &ChannelEndpoint
          }
        };

        channel.on_data(self.active_channel, text);
      }

      TelnetToken::Command(command) => {
        match command {
          SE => {
            self.active_channel = None;
          }

          _ => {}
        }

        let default = DefaultEndpoint;
        let channel = match self.commands.get_mut(&(command as uint)) {
          Some(channel) => { &**channel }
          None => { &default as &CommandEndpoint }
        };

        channel.on_command(None, command);
      }

      TelnetToken::Negotiation{command, channel} => {
        match command {
          WILL => {}
          WONT => {}
          DO => {}
          DONT => {}

          SB => {
            self.active_channel = Some(channel);
          }

          _ => {}
        }
        DefaultEndpoint.on_command(Some(channel), command);
      }
    }
  }
}
