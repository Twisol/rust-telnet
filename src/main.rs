#![feature(slicing_syntax, globs, unboxed_closures)]
#![allow(dead_code, unused_parens)]
use parser::{TelnetTokenizer};
use analyzer::{ChannelData, TelnetDemuxer};

mod parser;
mod analyzer;

fn main() {
  let stream = [b"\xFF\xFA\x20hello, w\xFF\xFForld!\xFF", b"\xF0"];

  let mut tokenizer = TelnetTokenizer::new();
  let mut demuxer = TelnetDemuxer::new();

  for &data in stream.iter() {
    for token in tokenizer.tokenize(data) {
      let packet = demuxer.analyze(token);
      println!("[{}]: {}", foo(&packet), &packet.message);
    }
  }
}

fn foo(datum: &ChannelData) -> String {
  match datum.channel {
    Some(channel) => { channel.to_string() }
    None => { String::from_str("M") }
  }
}
