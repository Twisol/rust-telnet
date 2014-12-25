use std::collections::{VecMap};
use std::cell::{RefCell};
use dispatch::{DataEndpoint, CommandEndpoint};
use qstate::{QState};

#[allow(non_snake_case)]
pub mod IAC {
  pub const SE: uint = 240;
  pub const SB: uint = 250;
  pub const WILL: uint = 251;
  pub const WONT: uint = 252;
  pub const DO: uint = 253;
  pub const DONT: uint = 254;
}


pub trait ChannelEndpoint {
  fn on_data<'a>(&mut self, _: Option<u8>, _: &'a [u8]) {}
  fn on_enable(&mut self, _: Option<u8>) {}
  fn on_disable(&mut self, _: Option<u8>) {}
  fn on_focus(&mut self, _: Option<u8>) {}
  fn on_blur(&mut self, _: Option<u8>) {}
}
pub trait PChannelEndpoint {
  fn _on_data<'a>(&self, _: Option<u8>, _: &'a [u8]) {}
  fn _on_enable(&self, _: Option<u8>) {}
  fn _on_disable(&self, _: Option<u8>) {}
  fn _on_focus(&self, _: Option<u8>) {}
  fn _on_blur(&self, _: Option<u8>) {}
}

impl PChannelEndpoint for () {}

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

static DEFAULT_ENDPOINT: () = ();

pub struct TelnetDemux<'a> {
  pub qstate: [QState, ..256],
  pub active_channel: Option<u8>,
  pub channels: VecMap<&'a (PChannelEndpoint + 'a)>,
  pub main_channel: &'a (PChannelEndpoint + 'a),
}
impl<'a> TelnetDemux<'a> {
  pub fn new() -> TelnetDemux<'a> {
    TelnetDemux {
      qstate: [QState::new(), ..256],
      active_channel: None,
      channels: VecMap::new(),
      main_channel: &DEFAULT_ENDPOINT,
    }
  }
}
impl<'a> CommandEndpoint for TelnetDemux<'a> {
  fn on_command(&mut self, channel: Option<u8>, command: u8) {
    match channel {
      None => {
        match command as uint {
          IAC::SE => {
            self.active_channel = channel;
            println!("IAC SE");
          },
          _  => {},
        }
      },
      Some(ch) => {
        match command as uint {
          IAC::WILL => println!("IAC WILL {}", ch),
          IAC::WONT => println!("IAC WONT {}", ch),
          IAC::DO   => println!("IAC DO {}", ch),
          IAC::DONT => println!("IAC DONT {}", ch),
          IAC::SB   => {
            self.active_channel = channel;
            println!("IAC SB {}", ch);
          },
          _    => {},
        }
      }
    };
  }
}
impl<'a> DataEndpoint for TelnetDemux<'a> {
  fn on_data<'b>(&mut self, data: &'b [u8]) {
    let endpoint = match self.active_channel {
      Some(ch) => {
        match self.channels.get(&(ch as uint)) {
          Some(endpoint) => { &**endpoint }
          None => { &DEFAULT_ENDPOINT as &PChannelEndpoint }
        }
      },
      None => { self.main_channel },
    };

    endpoint._on_data(self.active_channel, data);
  }
}
