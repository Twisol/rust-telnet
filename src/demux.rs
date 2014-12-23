use parser::{TelnetToken};
use std::collections::{VecMap};
pub use std::cell::{RefCell};
use qstate::{QState};

const SE: u8 = 240;
const SB: u8 = 250;
const WILL: u8 = 251;
const WONT: u8 = 252;
const DO: u8 = 253;
const DONT: u8 = 254;


pub trait ChannelEndpoint {
  fn on_data<'a>(&mut self, _: Option<u8>, _: &'a [u8]) {}
  fn on_enable(&mut self, _: Option<u8>) {}
  fn on_disable(&mut self, _: Option<u8>) {}
  fn on_focus(&mut self, _: Option<u8>) {}
  fn on_blur(&mut self, _: Option<u8>) {}
}
pub trait CommandEndpoint {
  fn on_command(&mut self, _: Option<u8>, _: u8) {}
}
pub trait NegotiableChannel: ChannelEndpoint {
  fn should_enable(&mut self, _: Option<u8>) -> bool { false }
}


pub trait PChannelEndpoint {
  fn _on_data<'a>(&self, _: Option<u8>, _: &'a [u8]) {}
  fn _on_enable(&self, _: Option<u8>) {}
  fn _on_disable(&self, _: Option<u8>) {}
  fn _on_focus(&self, _: Option<u8>) {}
  fn _on_blur(&self, _: Option<u8>) {}
}

impl<T> PChannelEndpoint for RefCell<T>
where T: ChannelEndpoint {
  fn _on_data<'a>(&self, channel: Option<u8>, data: &'a [u8]) {
    self.borrow_mut().on_data(channel, data);
  }
  fn _on_enable(&self, channel: Option<u8>) {
    self.borrow_mut().on_enable(channel);
  }
  fn _on_disable(&self, channel: Option<u8>) {
    self.borrow_mut().on_disable(channel);
  }
  fn _on_focus(&self, channel: Option<u8>) {
    self.borrow_mut().on_focus(channel);
  }
  fn _on_blur(&self, channel: Option<u8>) {
    self.borrow_mut().on_blur(channel);
  }
}


pub trait PCommandEndpoint {
  fn _on_command(&self, _: Option<u8>, _: u8) {}
}

impl<T> PCommandEndpoint for RefCell<T>
where T: CommandEndpoint {
  fn _on_command(&self, channel: Option<u8>, command: u8) {
    self.borrow_mut().on_command(channel, command);
  }
}


pub trait PNegotiableChannel: PChannelEndpoint {
  fn _should_enable(&self, _: Option<u8>) -> bool { false }
}

impl<T> PNegotiableChannel for RefCell<T>
where T: NegotiableChannel {
  fn _should_enable(&self, channel: Option<u8>) -> bool {
    self.borrow_mut().should_enable(channel)
  }
}


struct DefaultEndpoint;
impl PChannelEndpoint for DefaultEndpoint {
  fn _on_data<'a>(&self, channel: Option<u8>, data: &'a [u8]) {
    match channel {
      None     => println!("[{}]: {}", 'M', data),
      Some(ch) => println!("[{}]: {}", ch.to_string(), data),
    }
  }
}
impl PCommandEndpoint for DefaultEndpoint {
  fn _on_command(&self, channel: Option<u8>, cmd: u8) {
    match channel {
      None     => println!("IAC {}", cmd),
      Some(ch) => println!("IAC {} {}", cmd, ch),
    }
  }
}


pub struct TelnetDemux<'a> {
  pub active_channel: Option<u8>,
  pub qstate: QState,
  pub channels: VecMap<&'a (PChannelEndpoint + 'a)>,
  pub commands: VecMap<&'a (PCommandEndpoint + 'a)>,
}


impl<'a> TelnetDemux<'a> {
  pub fn new() -> TelnetDemux<'a> {
    TelnetDemux {
      qstate: QState::new(),
      active_channel: None,
      channels: VecMap::new(),
      commands: VecMap::new(),
    }
  }

  pub fn dispatch(&mut self, token: TelnetToken) {
    match token {
      TelnetToken::Text(text) => {
        let default = DefaultEndpoint;
        let channel = match self.active_channel {
          Some(ch) => {
            match self.channels.get_mut(&(ch as uint)) {
              Some(channel) => { &**channel }
              None => { &default as &PChannelEndpoint }
            }
          }
          None => {
            &default as &PChannelEndpoint
          }
        };

        channel._on_data(self.active_channel, text);
      }

      TelnetToken::Command(command) => {
        match command {
          SE => {
            self.active_channel = None;
          }

          _ => {}
        }

        let default = DefaultEndpoint;
        let channel = match self.commands.get_mut(&(command as uint)) {
          Some(channel) => { &**channel }
          None => { &default as &PCommandEndpoint }
        };

        channel._on_command(None, command);
      }

      TelnetToken::Negotiation{command, channel} => {
        match command {
          WILL => {}
          WONT => {}
          DO => {}
          DONT => {}

          SB => {
            self.active_channel = Some(channel);
          }

          _ => {}
        }
        DefaultEndpoint._on_command(Some(channel), command);
      }
    }
  }
}
