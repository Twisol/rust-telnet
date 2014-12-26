#![feature(slicing_syntax, globs, unboxed_closures)]
//#![allow(dead_code)]

use std::cell::{RefCell};
use parser::{TelnetTokenizer};
use dispatch::{TelnetDispatch, DataEndpoint, CommandEndpoint, TelnetDispatchVisitor};
use demux::{TelnetDemux, ChannelEndpoint, IAC};

mod parser;
mod dispatch;
mod demux;
mod qstate;


struct Foo(u8);
impl ChannelEndpoint for Foo {
  fn on_data<'b>(&mut self, _channel: Option<u8>, text: &'b [u8]) {
    self.0 += 1;
    println!("[FOO]: {} {}", self.0, text);
  }
  fn on_focus(&mut self, _channel: Option<u8>) {
    println!("[FOO] <focus>");
  }
  fn on_blur(&mut self, _channel: Option<u8>) {
    println!("[FOO] <blur>");
  }
}
impl CommandEndpoint for Foo {
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


impl DataEndpoint for () {}
impl CommandEndpoint for () {}
impl ChannelEndpoint for () {}
static mut DEFAULT_HANDLER: () = ();

struct Blargh<'a> {
  negotiator: TelnetDemux<'a>,
  foo: Foo,
}
impl<'b> TelnetDispatchVisitor for Blargh<'b> {
  fn data_handler<'a>(&'a mut self) -> &'a mut DataEndpoint {
    &mut self.negotiator
  }
  fn command_handler<'a>(&'a mut self, command: u8) -> &'a mut CommandEndpoint {
    match command {
      IAC::WILL | IAC::WONT | IAC::DO | IAC::DONT | IAC::SB | IAC::SE => {
        &mut self.negotiator
      }
      0x42 => {
        &mut self.foo
      }
      _ => {
        unsafe { &mut DEFAULT_HANDLER }
      }
    }
  }
}

fn main() {
  let stream = [b"abc", b"def\xFF\xFA\x20hello, w\xFF\xFForld!\xFF", b"\xF0\xFF\x42"];

  let mut tokenizer = TelnetTokenizer::new();
  let mut blargh = Blargh {
    negotiator: TelnetDemux::new(),
    foo: Foo(42),
  };

  //blargh.negotiator.channels.insert(32, &foo);

  let my_main = RefCell::new(Main);
  blargh.negotiator.main_channel = &my_main;

  for &data in stream.iter() {
    for token in tokenizer.tokenize(data) {
      (TelnetDispatch {context: &mut blargh}).dispatch(token);
    }
  }
}
