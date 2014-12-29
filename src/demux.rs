use dispatch::{DataEndpoint};
pub use dispatch::{CommandEndpoint};
use qstate::{QState, QAttitude};
use iac::{IAC};

pub trait ChannelEndpoint {
  fn on_data<'a>(&mut self, _: Option<u8>, _: &'a [u8]) {}
  fn on_enable(&mut self, _: Option<u8>) {}
  fn on_disable(&mut self, _: Option<u8>) {}
  fn on_focus(&mut self, _: Option<u8>) {}
  fn on_blur(&mut self, _: Option<u8>) {}

  fn should_enable(&mut self, _: Option<u8>, _: QAttitude) -> bool { false }
}

impl ChannelEndpoint for () {}


pub struct TelnetDemuxState {
  pub qstate: [QState, ..256],
  pub active_channel: Option<u8>,
}
impl TelnetDemuxState {
  pub fn new() -> TelnetDemuxState {
    TelnetDemuxState {
      qstate: [QState::new(), ..256],
      active_channel: None,
    }
  }
}

pub struct TelnetDemux<'a, Parent> {
  parent: Parent,

  state: &'a mut TelnetDemuxState,
}
impl<'b, Parent> TelnetDemux<'b, Parent> {
  pub fn new(state: &'b mut TelnetDemuxState, parent: Parent) -> TelnetDemux<'b, Parent> {
    TelnetDemux {
      state: state,
      parent: parent,
    }
  }
}
impl<'b, Parent> DataEndpoint for TelnetDemux<'b, Parent>
where Parent: ChannelEndpoint {
  fn on_data<'a>(&mut self, data: &'a [u8]) {
    self.parent.on_data(self.state.active_channel, data);
  }
}
impl<'b, Parent> CommandEndpoint for TelnetDemux<'b, Parent>
where Parent: CommandEndpoint + ChannelEndpoint {
  fn on_command(&mut self, channel: Option<u8>, command: u8) {
    match channel {
      None => {
        match command {
          IAC::SE => {
            self.parent.on_blur(self.state.active_channel);
            self.state.active_channel = channel;
          },
          _ => {
            self.parent.on_command(channel, command)
          },
        }
      },
      Some(ch) => {
        match command {
          IAC::WILL => println!("IAC WILL {}", ch),
          IAC::WONT => println!("IAC WONT {}", ch),
          IAC::DO   => println!("IAC DO {}", ch),
          IAC::DONT => println!("IAC DONT {}", ch),
          IAC::SB   => {
            self.parent.on_focus(channel);
            self.state.active_channel = channel;
          },
          _ => {
            self.parent.on_command(channel, command)
          },
        }
      }
    }
  }
}
