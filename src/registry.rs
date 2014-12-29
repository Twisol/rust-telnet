use std::collections::{HashMap};
use std::vec::{Vec};
use demux::{ChannelEndpoint, CommandEndpoint};
use qstate::{QAttitude};

pub trait TelnetEndpoint {
  fn on_data<'a>(&mut self, _: Option<u8>, _: &'a [u8]) {}
  fn on_command(&mut self, _: Option<u8>, _: u8) {}

  fn on_enable(&mut self, _: Option<u8>) {}
  fn on_disable(&mut self, _: Option<u8>) {}
  fn on_focus(&mut self, _: Option<u8>) {}
  fn on_blur(&mut self, _: Option<u8>) {}

  fn should_enable(&mut self, _: Option<u8>, _: QAttitude) -> bool { false }
}
impl TelnetEndpoint for () {}


pub struct EndpointRegistry<'b, Parent> {
  pub parent: Parent,

  pub command_map: HashMap<u8, uint>,
  pub channel_map: HashMap<u8, uint>,
  pub endpoints: Vec<&'b mut (TelnetEndpoint + 'b)>,

  pub main: Option<&'b mut (TelnetEndpoint + 'b)>,
}
impl<'b, Parent> EndpointRegistry<'b, Parent>
where Parent: TelnetEndpoint {
  pub fn new(parent: Parent) -> EndpointRegistry<'b, Parent> {
    EndpointRegistry {
      command_map: HashMap::new(),
      channel_map: HashMap::new(),
      endpoints: Vec::new(),

      main: None,
      parent: parent,
    }
  }

  fn _get_command_handler<'a>(&'a mut self, command: u8) -> &'a mut TelnetEndpoint {
    match self.command_map.get(&command) {
      Some(&id) => *self.endpoints.get_mut(id).unwrap(),
      None => &mut self.parent,
    }
  }

  fn _get_channel_handler<'a>(&'a mut self, channel: Option<u8>) -> &'a mut TelnetEndpoint {
    match channel {
      None => {
        match self.main {
          Some(ref mut endpoint) => *endpoint,
          None => &mut self.parent,
        }
      },
      Some(ch) => {
        match self.channel_map.get(&ch) {
          Some(&id) => *self.endpoints.get_mut(id).unwrap(),
          None => &mut self.parent,
        }
      }
    }
  }
}

impl<'b, Parent> CommandEndpoint for EndpointRegistry<'b, Parent>
where Parent: TelnetEndpoint {
  fn on_command(&mut self, channel: Option<u8>, command: u8) {
    self._get_command_handler(command).on_command(channel, command);
  }
}
impl<'b, Parent> ChannelEndpoint for EndpointRegistry<'b, Parent>
where Parent: TelnetEndpoint {
  fn on_data<'a>(&mut self, channel: Option<u8>, data: &'a [u8]) {
    self._get_channel_handler(channel).on_data(channel, data);
  }
  fn on_enable(&mut self, channel: Option<u8>) {
    self._get_channel_handler(channel).on_enable(channel);
  }
  fn on_disable(&mut self, channel: Option<u8>) {
    self._get_channel_handler(channel).on_disable(channel);
  }
  fn on_focus(&mut self, channel: Option<u8>) {
    self._get_channel_handler(channel).on_focus(channel);
  }
  fn on_blur(&mut self, channel: Option<u8>) {
    self._get_channel_handler(channel).on_blur(channel);
  }
  fn should_enable(&mut self, channel: Option<u8>, attitude: QAttitude) -> bool {
    self._get_channel_handler(channel).should_enable(channel, attitude)
  }
}
