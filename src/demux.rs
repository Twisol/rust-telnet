use carrier::{Carrier};
use dispatch::{DispatchHandler};
use qstate::{QState, QAttitude};
use iac::{IAC};

pub trait ChannelHandler {
  fn on_data<'a>(&mut self, _channel: Option<u8>, _data: &'a [u8]) {}
  fn on_command(&mut self, _channel: Option<u8>, _command: u8) {}

  fn on_enable(&mut self, _channel: Option<u8>) {}
  fn on_disable(&mut self, _channel: Option<u8>) {}
  fn on_focus(&mut self, _channel: Option<u8>) {}
  fn on_blur(&mut self, _channel: Option<u8>) {}

  fn should_enable(&mut self, _channel: Option<u8>, _attitude: QAttitude) -> bool { false }
}

impl ChannelHandler for () {}


#[derive(Copy)]
pub struct TelnetDemuxState {
  pub qstate: [QState; 256],
  pub active_channel: Option<u8>,
}
impl TelnetDemuxState {
  pub fn new() -> TelnetDemuxState {
    TelnetDemuxState {
      qstate: [QState::new(); 256],
      active_channel: None,
    }
  }
}

impl<'state, 'parent, Parent> DispatchHandler for Carrier<'state, 'parent, Parent, TelnetDemuxState>
where Parent: ChannelHandler {
  fn on_data<'a>(&mut self, data: &'a [u8]) {
    self.parent.on_data(self.state.active_channel, data);
  }

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
