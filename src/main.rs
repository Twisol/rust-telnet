#![feature(slicing_syntax, globs)]
#![allow(dead_code, unused_parens)]
use parser::{ParseState, TelnetToken, TelnetTokenizer};
mod parser;

#[deriving(Copy)]
struct QState {
  status: [u8, ..256]
}

impl QState {
  fn new() -> QState {
    return QState{status: [0, ..256]}
  }
}

#[deriving(Copy)]
struct TelnetDemuxer {
  active_channel: Option<u8>,
  qstate: QState,
}

#[deriving(Show, Copy)]
enum TelnetPacket<'a> {
  Text(&'a [u8]),
  Command(u8),
}

#[deriving(Show, Copy)]
struct ChannelData<'a> {
  channel: Option<u8>,
  message: TelnetPacket<'a>,
}

fn analyze<'a>(state: TelnetDemuxer, token: TelnetToken<'a>) -> (TelnetDemuxer, ChannelData<'a>) {
  match token {
    TelnetToken::Text(data) => {
      (state, ChannelData{channel: state.active_channel, message: TelnetPacket::Text(data)})
    }
    TelnetToken::Command(command) => {
      let newstate = (if command == 0xF0 {
        TelnetDemuxer {active_channel: None, ..state}
      } else {
        state
      });
      (newstate, ChannelData{channel: None, message: TelnetPacket::Command(command)})
    }
    TelnetToken::Negotiation{command, channel} => {
      let newstate = (if command == 0xFA {
        TelnetDemuxer {active_channel: Some(channel), ..state}
      } else {
        state
      });
      (newstate, ChannelData{channel: Some(channel), message: TelnetPacket::Command(command)})
    }
  }
}

fn main() {
  let tokenizer = TelnetTokenizer {
    state: ParseState::Neutral,
    data: b"\xFF\xFA\x20hello, world!\xFF\xF0"
  };

  let demuxer = TelnetDemuxer {
    active_channel: None,
    qstate: QState::new(),
  };

  let mut iter = tokenizer.map(|token| analyze(demuxer, token));
  while let Some(packet) = iter.next() {
    println!("[{}]: {}", foo(&packet.1), &packet.1.message);
  }
}

fn foo(datum: &ChannelData) -> String {
  match datum.channel {
    Some(channel) => { channel.to_string() }
    None => { String::from_str("M") }
  }
}
