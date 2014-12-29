#![feature(slicing_syntax, globs, unboxed_closures)]
//#![allow(dead_code)]

use parser::{TelnetTokenizer};
use dispatch::{DispatchExt};
use demux::{TelnetDemuxState, TelnetDemux};
use registry::{EndpointRegistry, ChannelHandler, SomeThing};

mod parser;
mod dispatch;
mod demux;
mod registry;

mod qstate;
mod iac;

trait MyWritable {
  fn mywrite(&mut self, _s: String);
}

struct Foo(u8);
impl<Parent> SomeThing<Parent> for Foo
where Parent: MyWritable {
  fn visit(&mut self, parent: &mut Parent, scope: &Fn(&mut ChannelHandler)) {
    scope.call((&mut FooThing {
      parent: parent,
      state: self,
    },))
  }
  fn ask(&mut self, parent: &mut Parent, scope: &Fn(&mut ChannelHandler) -> bool) -> bool {
    scope.call((&mut FooThing {
      parent: parent,
      state: self,
    },))
  }
}

struct FooThing<'state, 'parent, Parent: 'parent> {
  parent: &'parent mut Parent,
  state: &'state mut Foo,
}
impl<'state, 'parent, Parent> ChannelHandler for FooThing<'state, 'parent, Parent>
where Parent: MyWritable {
  fn on_data<'b>(&mut self, _channel: Option<u8>, text: &'b [u8]) {
    self.state.0 += 1;
    self.parent.mywrite(format!("[FOO]: {} {}", self.state.0, text));
  }
  fn on_command(&mut self, _channel: Option<u8>, _command: u8) {
    self.state.0 += 1;
    self.parent.mywrite(format!("TEST TEST {}", self.state.0));
  }

  fn on_focus(&mut self, _channel: Option<u8>) {
    self.parent.mywrite(format!("[FOO] <focus>"));
  }
  fn on_blur(&mut self, _channel: Option<u8>) {
    self.parent.mywrite(format!("[FOO] <blur>"));
  }
}


struct Main;
impl ChannelHandler for Main {
  fn on_data<'b>(&mut self, _channel: Option<u8>, text: &'b [u8]) {
    println!("[M]: {}", text);
  }
}


struct Output {
  out: String,
}
impl MyWritable for Output {
  fn mywrite(&mut self, s: String) {
    self.out.push_str(&*s);
    self.out.push_str("\r\n");
  }
}
impl ChannelHandler for Output {}

fn main() {
  let stream = [
    b"abcdef\xFF\xFA\x20h",
    b"ello",
    b", world!\xFF\xF0\xFF\x42\xFF\xFE\x42"
  ];

  let mut output = Output {
    out: String::new(),
  };

  let mut tokenizer = TelnetTokenizer::new();

  let mut demux = TelnetDemuxState::new();
  let mut foo = Foo(42);
  let mut main_channel = Main;

  for &data in stream.iter() {
    for token in tokenizer.tokenize(data) {
      // Construct an event context
      let mut registry = EndpointRegistry::new(&mut output);
      registry.main = Some(&mut main_channel as &mut ChannelHandler);
      registry.endpoints.push(&mut foo);
      registry.command_map.insert(0x42, registry.endpoints.len() - 1);
      registry.channel_map.insert(32, registry.endpoints.len() - 1);

      let mut demux = TelnetDemux::new(&mut demux, &mut registry);

      // Dispatch the event
      demux.dispatch(token);
    }
  }

  println!("\r\nBuffered output:\r\n{}", output.out);
}
