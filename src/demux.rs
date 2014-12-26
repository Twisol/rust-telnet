use std::collections::{VecMap};
use std::cell::{RefCell};
use dispatch::{DataEndpoint, CommandEndpoint};
use qstate::{QState, QAttitude};

#[allow(non_snake_case)]
pub mod IAC {
  pub const SE: u8 = 240;
  pub const SB: u8 = 250;
  pub const WILL: u8 = 251;
  pub const WONT: u8 = 252;
  pub const DO: u8 = 253;
  pub const DONT: u8 = 254;
}

pub trait ChannelEndpoint {
  fn on_data<'a>(&mut self, _: Option<u8>, _: &'a [u8]) {}
  fn on_enable(&mut self, _: Option<u8>) {}
  fn on_disable(&mut self, _: Option<u8>) {}
  fn on_focus(&mut self, _: Option<u8>) {}
  fn on_blur(&mut self, _: Option<u8>) {}

  fn should_enable(&mut self, _: QAttitude) -> bool { false }
}
pub trait PChannelEndpoint {
  fn _on_data<'a>(&self, _: Option<u8>, _: &'a [u8]) {}
  fn _on_enable(&self, _: Option<u8>) {}
  fn _on_disable(&self, _: Option<u8>) {}
  fn _on_focus(&self, _: Option<u8>) {}
  fn _on_blur(&self, _: Option<u8>) {}

  fn _should_enable(&self, _: QAttitude) -> bool { false }
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

  fn _should_enable(&self, attitude: QAttitude) -> bool {
    self.borrow_mut().should_enable(attitude)
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
        let endpoint = match self.active_channel {
          Some(ch) => {
            match self.channels.get(&(ch as uint)) {
              Some(endpoint) => { &**endpoint }
              None => { &DEFAULT_ENDPOINT as &PChannelEndpoint }
            }
          },
          None => { &DEFAULT_ENDPOINT as &PChannelEndpoint },
        };

        match command {
          IAC::SE => {
            endpoint._on_blur(self.active_channel);
            self.active_channel = channel;
          },
          _  => {},
        }
      },
      Some(ch) => {
        let endpoint = match self.channels.get(&(ch as uint)) {
          Some(endpoint) => { &**endpoint }
          None => { &DEFAULT_ENDPOINT as &PChannelEndpoint }
        };

        match command {
          IAC::WILL => println!("IAC WILL {}", ch),
          IAC::WONT => println!("IAC WONT {}", ch),
          IAC::DO   => println!("IAC DO {}", ch),
          IAC::DONT => println!("IAC DONT {}", ch),
          IAC::SB   => {
            let ref qstate = self.qstate[ch as uint];
            if true { //qstate.is_active(QAttitude::Local) || qstate.is_active(QAttitude::Remote) {
              self.active_channel = channel;
              endpoint._on_focus(self.active_channel);
            }
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
