use dispatch::{DataEndpoint, CommandEndpoint};
use qstate::{QState, QAttitude};
use iac::IAC;

pub trait ChannelEndpoint {
  fn on_data<'a>(&mut self, _: Option<u8>, _: &'a [u8]) {}
  fn on_enable(&mut self, _: Option<u8>) {}
  fn on_disable(&mut self, _: Option<u8>) {}
  fn on_focus(&mut self, _: Option<u8>) {}
  fn on_blur(&mut self, _: Option<u8>) {}

  fn should_enable(&mut self, _: QAttitude) -> bool { false }
}
pub trait TelnetDemuxVisitor {
  fn channel_handler(&mut self, _channel: Option<u8>, scope: &Fn(&mut ChannelEndpoint)) {
    scope.call((&mut (),));
  }
}

impl ChannelEndpoint for () {}
impl TelnetDemuxVisitor for () {}


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

  pub fn visit<'a>(&'a mut self, visitor: &'a mut (TelnetDemuxVisitor + 'a)) -> TelnetDemux<'a> {
    TelnetDemux {
      context: visitor,
      state: self,
    }
  }
}

pub struct TelnetDemux<'a> {
  context: &'a mut (TelnetDemuxVisitor + 'a),
  state: &'a mut TelnetDemuxState,
}
impl<'a> CommandEndpoint for TelnetDemux<'a> {
  fn on_command(&mut self, channel: Option<u8>, command: u8) {
    match channel {
      None => {
        match command {
          IAC::SE => {
            let prev_channel = self.state.active_channel;
            self.state.active_channel = channel;

            self.context.channel_handler(prev_channel, &|handler| {
              handler.on_blur(prev_channel);
            });
          },
          _ => {},
        }
      },
      Some(ch) => {
        match command {
          IAC::WILL => println!("IAC WILL {}", ch),
          IAC::WONT => println!("IAC WONT {}", ch),
          IAC::DO   => println!("IAC DO {}", ch),
          IAC::DONT => println!("IAC DONT {}", ch),
          IAC::SB   => {
            self.state.active_channel = channel;
            self.context.channel_handler(channel, &|handler| {
              handler.on_focus(channel);
            });
          },
          _ => {},
        }
      }
    };
  }
}
impl<'a> DataEndpoint for TelnetDemux<'a> {
  fn on_data<'b>(&mut self, data: &'b [u8]) {
    let active_channel = self.state.active_channel;
    self.context.channel_handler(active_channel, &|handler| {
      handler.on_data(active_channel, data);
    });
  }
}
