use parser::{TelnetToken};

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
pub struct TelnetDemuxer {
  active_channel: Option<u8>,
  qstate: QState,
}

#[deriving(Show, Copy)]
pub enum TelnetPacket<'a> {
  Text(&'a [u8]),
  Command(u8),
}

#[deriving(Show, Copy)]
pub struct ChannelData<'a> {
  pub channel: Option<u8>,
  pub message: TelnetPacket<'a>,
}

impl TelnetDemuxer {
  pub fn new() -> TelnetDemuxer {
    TelnetDemuxer {
      active_channel: None,
      qstate: QState::new(),
    }
  }

  pub fn analyze<'a>(&mut self, token: TelnetToken<'a>) -> ChannelData<'a> {
    match token {
      TelnetToken::Text(data) => {
        ChannelData{channel: self.active_channel, message: TelnetPacket::Text(data)}
      }
      TelnetToken::Command(command) => {
        if command == 0xF0 {
          self.active_channel = None;
        }
        ChannelData{channel: None, message: TelnetPacket::Command(command)}
      }
      TelnetToken::Negotiation{command, channel} => {
        if command == 0xFA {
          self.active_channel = Some(channel);
        }
        ChannelData{channel: Some(channel), message: TelnetPacket::Command(command)}
      }
    }
  }
}
