#![feature(slicing_syntax, globs, unboxed_closures)]
//#![allow(dead_code)]

use std::cell::{RefCell};
use parser::{TelnetTokenizer};
use dispatch::{TelnetDispatch, DataEndpoint, CommandEndpoint, TelnetDispatchVisitor};
use demux::{TelnetDemux, ChannelEndpoint, IAC, TelnetDemuxVisitor, TelnetDemuxState};

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

struct Blargh<'a> {
  negotiator: &'a mut TelnetDemuxState,
  foo: &'a mut Foo,
  main: &'a mut Main,
  default_handler: (),
}
impl<'a> TelnetDispatchVisitor for Blargh<'a> {
  fn data_handler(&mut self, scope: &Fn(&mut DataEndpoint)) {
    let mut demux = TelnetDemux {
      context: &mut Blargh2 {
        foo: self.foo,
        main: self.main,
        default_handler: (),
      },
      state: self.negotiator,
    };
    scope.call((&mut demux,));
  }
  fn command_handler(&mut self, command: u8, scope: &Fn(&mut CommandEndpoint)) {
    match command {
      IAC::WILL | IAC::WONT | IAC::DO | IAC::DONT | IAC::SB | IAC::SE => {
        let mut demux = TelnetDemux {
          context: &mut Blargh2 {
            foo: self.foo,
            main: self.main,
            default_handler: (),
          },
          state: self.negotiator,
        };
        scope.call((&mut demux,));
      }
      0x42 => {
        scope.call((self.foo,));
      }
      _ => {
        scope.call((&mut self.default_handler,));
      }
    }
  }
}

struct Blargh2<'a> {
  foo: &'a mut Foo,
  main: &'a mut Main,
  default_handler: (),
}
impl<'a> TelnetDemuxVisitor for Blargh2<'a> {
  fn channel_handler(&mut self, channel: Option<u8>, scope: &Fn(&mut ChannelEndpoint)) {
    match channel {
      None => {
        scope.call((self.main,));
      },
      Some(32) => {
        scope.call((self.foo,));
      },
      Some(_) => {
        scope.call((&mut self.default_handler,));
      },
    }
  }
}

fn main() {
  let stream = [b"abc", b"def\xFF\xFA\x20hello, w\xFF\xFForld!\xFF", b"\xF0\xFF\x42"];

  let mut tokenizer = TelnetTokenizer::new();

  let mut negotiator = TelnetDemuxState::new();
  let mut foo = Foo(42);
  let mut mainc = Main;

  let mut blargh = Blargh {
    negotiator: &mut negotiator,
    foo: &mut foo,
    main: &mut mainc,
    default_handler: (),
  };

  for &data in stream.iter() {
    for token in tokenizer.tokenize(data) {
      (TelnetDispatch {context: &mut blargh}).dispatch(token);
    }
  }
}
