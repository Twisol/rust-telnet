use std::collections::{HashMap};
use std::vec::{Vec};
pub use demux::{ChannelHandler};
use qstate::{QAttitude};

pub struct EndpointRegistry<'b, Parent> {
  pub parent: Parent,

  pub command_map: HashMap<u8, uint>,
  pub channel_map: HashMap<u8, uint>,
  pub endpoints: Vec<&'b mut (ChannelHandler + 'b)>,

  pub main: Option<&'b mut (ChannelHandler + 'b)>,
}
impl<'b, Parent> EndpointRegistry<'b, Parent>
where Parent: ChannelHandler {
  pub fn new(parent: Parent) -> EndpointRegistry<'b, Parent> {
    EndpointRegistry {
      command_map: HashMap::new(),
      channel_map: HashMap::new(),
      endpoints: Vec::new(),

      main: None,
      parent: parent,
    }
  }

  fn _get_command_handler<'a>(&'a mut self, command: u8) -> &'a mut ChannelHandler {
    match self.command_map.get(&command) {
      Some(&id) => *self.endpoints.get_mut(id).unwrap(),
      None => &mut self.parent,
    }
  }

  fn _get_channel_handler<'a>(&'a mut self, channel: Option<u8>) -> &'a mut ChannelHandler {
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

impl<'b, Parent> ChannelHandler for EndpointRegistry<'b, Parent>
where Parent: ChannelHandler {
  fn on_data<'a>(&mut self, channel: Option<u8>, data: &'a [u8]) {
    self._get_channel_handler(channel).on_data(channel, data);
  }
  fn on_command(&mut self, channel: Option<u8>, command: u8) {
    self._get_command_handler(command).on_command(channel, command);
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
