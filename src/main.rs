#![feature(slicing_syntax, globs, unboxed_closures)]
//#![allow(dead_code)]

use parser::{TelnetTokenizer};
use dispatch::{DispatchExt};
use demux::{TelnetDemuxState, TelnetDemux};
use registry::{EndpointRegistry, ChannelHandler};

mod parser;
mod dispatch;
mod demux;
mod registry;

mod qstate;
mod iac;



struct Foo(u8);
impl ChannelHandler for Foo {
  fn on_data<'b>(&mut self, _channel: Option<u8>, text: &'b [u8]) {
    self.0 += 1;
    println!("[FOO]: {} {}", self.0, text);
  }
  fn on_command(&mut self, _channel: Option<u8>, _command: u8) {
    self.0 += 1;
    println!("TEST TEST {}", self.0);
  }

  fn on_focus(&mut self, _channel: Option<u8>) {
    println!("[FOO] <focus>");
  }
  fn on_blur(&mut self, _channel: Option<u8>) {
    println!("[FOO] <blur>");
  }
}


struct Main;
impl ChannelHandler for Main {
  fn on_data<'b>(&mut self, _channel: Option<u8>, text: &'b [u8]) {
    println!("[M]: {}", text);
  }
}

fn main() {
  let stream = [
    b"abcdef\xFF\xFA\x20h",
    b"ello",
    b", world!\xFF\xF0\xFF\x42\xFF\xFE\x42"
  ];

  let mut tokenizer = TelnetTokenizer::new();

  let mut demux = TelnetDemuxState::new();
  let mut foo = Foo(42);
  let mut main_channel = Main;

  for &data in stream.iter() {
    for token in tokenizer.tokenize(data) {
      // Construct an event context

      let mut registry = EndpointRegistry::new(());
      registry.main = Some(&mut main_channel as &mut ChannelHandler);
      registry.endpoints.push(&mut foo);
      registry.command_map.insert(0x42, registry.endpoints.len() - 1);
      registry.channel_map.insert(32, registry.endpoints.len() - 1);

      let mut demux = TelnetDemux::new(&mut demux, registry);

      // Dispatch the event
      demux.dispatch(token);
    }
  }
}
