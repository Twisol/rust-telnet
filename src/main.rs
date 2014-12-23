#![feature(slicing_syntax, globs, unboxed_closures, macro_rules)]
//#![allow(dead_code, unused_parens)]
use parser::{TelnetTokenizer};
use demux::{TelnetDemux, ChannelEndpoint, CommandEndpoint, RefCell};

mod parser;
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


fn main() {
  let stream = [b"abc", b"def\xFF\xFA\x20hello, w\xFF\xFForld!\xFF", b"\xF0"];

  let mut tokenizer = TelnetTokenizer::new();
  let mut context = TelnetDemux::new();
  let foo = RefCell::new(Bar(42));
  context.channels.insert(32, &foo);
  context.commands.insert(240, &foo);

  for &data in stream.iter() {
    for token in tokenizer.tokenize(data) {
      context.dispatch(token);
    }
  }
}
