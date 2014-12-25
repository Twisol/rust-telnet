#![feature(slicing_syntax, globs, unboxed_closures)]
//#![allow(dead_code)]

use std::cell::{RefCell};
use parser::{TelnetTokenizer};
use dispatch::{TelnetDispatch, CommandEndpoint};
use demux::{TelnetDemux, ChannelEndpoint, IAC};

mod parser;
mod dispatch;
mod demux;
mod qstate;


struct Bar(u8);
impl ChannelEndpoint for Bar {
  fn on_data<'b>(&mut self, _channel: Option<u8>, text: &'b [u8]) {
    self.0 += 1;
    println!("[FOO]: {} {}", self.0, text);
  }
}
impl CommandEndpoint for Bar {
  fn on_command(&mut self, _channel: Option<u8>, _command: u8) {
    self.0 += 1;
    println!("TEST TEST {}", self.0);
  }
}


struct Main;
impl ChannelEndpoint for Main {
  fn on_data<'b>(&mut self, _channel: Option<u8>, text: &'b [u8]) {
    println!("[M]: {}", text);
  }
}


fn main() {
  let stream = [b"abc", b"def\xFF\xFA\x20hello, w\xFF\xFForld!\xFF", b"\xF0\xFF\x42"];

  let mut tokenizer = TelnetTokenizer::new();
  let mut dispatch = TelnetDispatch::new();

  let negotiator = RefCell::new(TelnetDemux::new());
  dispatch.data = &negotiator;
  dispatch.commands.insert(IAC::WILL, &negotiator);
  dispatch.commands.insert(IAC::WONT, &negotiator);
  dispatch.commands.insert(IAC::DO, &negotiator);
  dispatch.commands.insert(IAC::DONT, &negotiator);
  dispatch.commands.insert(IAC::SB, &negotiator);
  dispatch.commands.insert(IAC::SE, &negotiator);

  let bar = RefCell::new(Bar(42));
  negotiator.borrow_mut().channels.insert(32, &bar);
  dispatch.commands.insert(0x42, &bar);

  let my_main = RefCell::new(Main);
  negotiator.borrow_mut().main_channel = &my_main;

  for &data in stream.iter() {
    for token in tokenizer.tokenize(data) {
      dispatch.dispatch(token);
    }
  }
}
