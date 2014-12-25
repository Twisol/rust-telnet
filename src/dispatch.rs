use parser::{TelnetToken};
use std::collections::{VecMap};
use std::cell::{RefCell};


pub trait DataEndpoint {
  fn on_data<'a>(&mut self, _: &'a [u8]) {}
}
pub trait PDataEndpoint {
  fn _on_data<'a>(&self, _: &'a [u8]) {}
}

impl PDataEndpoint for () {}

impl<T> PDataEndpoint for RefCell<T>
where T: DataEndpoint {
  fn _on_data<'a>(&self, data: &'a [u8]) {
    self.borrow_mut().on_data(data);
  }
}


pub trait CommandEndpoint {
  fn on_command(&mut self, _: Option<u8>, _: u8) {}
}
pub trait PCommandEndpoint {
  fn _on_command(&self, _: Option<u8>, _: u8) {}
}

impl PCommandEndpoint for () {}

impl<T> PCommandEndpoint for RefCell<T>
where T: CommandEndpoint {
  fn _on_command(&self, channel: Option<u8>, command: u8) {
    self.borrow_mut().on_command(channel, command);
  }
}



static DEFAULT_ENDPOINT: () = ();

pub struct TelnetDispatch<'a> {
  pub data: &'a (PDataEndpoint + 'a),
  pub commands: VecMap<&'a (PCommandEndpoint + 'a)>,
}

impl<'a> TelnetDispatch<'a> {
  pub fn new() -> TelnetDispatch<'a> {
    TelnetDispatch {
      data: &DEFAULT_ENDPOINT,
      commands: VecMap::new(),
    }
  }

  pub fn dispatch(&mut self, token: TelnetToken) {
    match token {
      TelnetToken::Text(text) => {
        self.data._on_data(text);
      }

      TelnetToken::Command(command) => {
        let endpoint = match self.commands.get(&(command as uint)) {
          Some(endpoint) => { &**endpoint }
          None => { &DEFAULT_ENDPOINT as &PCommandEndpoint }
        };

        endpoint._on_command(None, command);
      }

      TelnetToken::Negotiation{command, channel} => {
        let endpoint = match self.commands.get(&(command as uint)) {
          Some(endpoint) => { &**endpoint }
          None => { &DEFAULT_ENDPOINT as &PCommandEndpoint }
        };

        endpoint._on_command(Some(channel), command);
      }
    }
  }
}
