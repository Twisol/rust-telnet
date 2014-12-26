#![feature(slicing_syntax, globs, unboxed_closures)]
//#![allow(dead_code)]

use parser::{TelnetTokenizer};
use dispatch::{TelnetDispatch, DataEndpoint, CommandEndpoint, TelnetDataVisitor, TelnetCommandVisitor};
use demux::{TelnetDemuxState, ChannelEndpoint, TelnetDemuxVisitor};
use iac::{IAC};

mod parser;
mod dispatch;
mod demux;
mod qstate;
mod iac;


struct Blargh<'a, Parent> {
  parent: Parent,

  negotiator: &'a mut TelnetDemuxState,
}
impl<'a, Parent> TelnetDataVisitor for Blargh<'a, Parent>
where Parent: TelnetDemuxVisitor {
  fn data_handler(&mut self, scope: &Fn(&mut DataEndpoint)) {
    scope.call((&mut self.negotiator.visit(&mut self.parent),));
  }
}
impl<'a, Parent> TelnetCommandVisitor for Blargh<'a, Parent>
where Parent: TelnetCommandVisitor + TelnetDemuxVisitor {
  fn command_handler(&mut self, command: u8, scope: &Fn(&mut CommandEndpoint)) {
    match command {
      IAC::WILL | IAC::WONT | IAC::DO | IAC::DONT | IAC::SB | IAC::SE => {
        scope.call((&mut self.negotiator.visit(&mut self.parent),));
      }
      _ => {
        self.parent.command_handler(command, scope);
      }
    }
  }
}


struct EndpointRegistry<'a> {
  foo: &'a mut Foo,
  main: &'a mut Main,
}
impl<'a> TelnetCommandVisitor for EndpointRegistry<'a> {
  fn command_handler(&mut self, command: u8, scope: &Fn(&mut CommandEndpoint)) {
    match command {
      0x42 => scope.call((self.foo,)),
      _    => scope.call((&mut (),)),
    }
  }
}
impl<'a> TelnetDemuxVisitor for EndpointRegistry<'a> {
  fn channel_handler(&mut self, channel: Option<u8>, scope: &Fn(&mut ChannelEndpoint)) {
    match channel {
      None     => scope.call((self.main,)),
      Some(32) => scope.call((self.foo,)),
      Some(_)  => scope.call((&mut (),)),
    }
  }
}

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


fn main() {
  let stream = [b"abcdef\xFF\xFA\x20hello, world!\xFF\xF0\xFF\x42\xFF\xFE\x42"];

  let mut tokenizer = TelnetTokenizer::new();

  let mut negotiator = TelnetDemuxState::new();
  let mut foo = Foo(42);
  let mut main_channel = Main;

  let mut blargh = Blargh {
    negotiator: &mut negotiator,

    parent: EndpointRegistry {
      foo: &mut foo,
      main: &mut main_channel,
    }
  };

  for &data in stream.iter() {
    for token in tokenizer.tokenize(data) {
      (TelnetDispatch {context: &mut blargh}).dispatch(token);
    }
  }
}
