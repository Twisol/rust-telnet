use std::collections::{HashMap};
use std::vec::{Vec};
pub use demux::{ChannelHandler};
use qstate::{QAttitude};

pub trait TelnetChannel<Parent> {
  fn on_data<'a>(&mut self, _parent: &mut Parent, _channel: Option<u8>, _data: &'a [u8]) {}
  fn on_command(&mut self, _parent: &mut Parent, _channel: Option<u8>, _command: u8) {}

  fn on_enable(&mut self, _parent: &mut Parent, _channel: Option<u8>) {}
  fn on_disable(&mut self, _parent: &mut Parent, _channel: Option<u8>) {}
  fn on_focus(&mut self, _parent: &mut Parent, _channel: Option<u8>) {}
  fn on_blur(&mut self, _parent: &mut Parent, _channel: Option<u8>) {}

  fn should_enable(&mut self, _parent: &mut Parent, _channel: Option<u8>, _attitude: QAttitude) -> bool { false }
}

impl<Parent> TelnetChannel<Parent> for ()
where Parent: ChannelHandler {
  fn on_data<'a>(&mut self, parent: &mut Parent, channel: Option<u8>, data: &'a [u8]) {
    parent.on_data(channel, data)
  }
  fn on_command(&mut self, parent: &mut Parent, channel: Option<u8>, command: u8) {
    parent.on_command(channel, command)
  }

  fn on_enable(&mut self, parent: &mut Parent, channel: Option<u8>) {
    parent.on_enable(channel)
  }
  fn on_disable(&mut self, parent: &mut Parent, channel: Option<u8>) {
    parent.on_disable(channel)
  }
  fn on_focus(&mut self, parent: &mut Parent, channel: Option<u8>) {
    parent.on_focus(channel)
  }
  fn on_blur(&mut self, parent: &mut Parent, channel: Option<u8>) {
    parent.on_blur(channel)
  }

  fn should_enable(&mut self, parent: &mut Parent, channel: Option<u8>, attitude: QAttitude) -> bool {
    parent.should_enable(channel, attitude)
  }
}


pub struct EndpointRegistry<'parent, Parent: 'parent> {
  pub parent: &'parent mut Parent,

  pub command_map: HashMap<u8, uint>,
  pub channel_map: HashMap<u8, uint>,
  pub endpoints: Vec<&'parent mut (TelnetChannel<Parent> + 'parent)>,

  pub main: Option<&'parent mut (TelnetChannel<Parent> + 'parent)>,
}
impl<'parent, Parent> EndpointRegistry<'parent, Parent>
where Parent: ChannelHandler {
  pub fn new(parent: &'parent mut Parent) -> EndpointRegistry<'parent, Parent> {
    EndpointRegistry {
      command_map: HashMap::new(),
      channel_map: HashMap::new(),
      endpoints: Vec::new(),

      main: None,
      parent: parent,
    }
  }
}


// I'd love to factor these match blocks out into their own functions,
//   but ownership (getting multiple fields from `self`) makes it hard.
// Also, (&mut ()) doesn't live long enough.
impl<'parent, Parent> ChannelHandler for EndpointRegistry<'parent, Parent>
where Parent: ChannelHandler {
  fn on_data<'a>(&mut self, channel: Option<u8>, data: &'a [u8]) {
    match channel {
      None => {
        match self.main {
          Some(ref mut endpoint) => (*endpoint).on_data(self.parent, channel, data),
          None => (&mut () as &mut TelnetChannel<Parent>).on_data(self.parent, channel, data),
        }
      },
      Some(ch) => {
        match self.channel_map.get(&ch) {
          Some(&id) => (*self.endpoints.get_mut(id).unwrap()).on_data(self.parent, channel, data),
          None => (&mut () as &mut TelnetChannel<Parent>).on_data(self.parent, channel, data),
        }
      }
    }
  }
  fn on_command(&mut self, channel: Option<u8>, command: u8) {
    match self.command_map.get(&command) {
      Some(&id) => (*self.endpoints.get_mut(id).unwrap()).on_command(self.parent, channel, command),
      None => (&mut () as &mut TelnetChannel<Parent>).on_command(self.parent, channel, command),
    }
  }

  fn on_enable(&mut self, channel: Option<u8>) {
    match channel {
      None => {
        match self.main {
          Some(ref mut endpoint) => (*endpoint).on_enable(self.parent, channel),
          None => (&mut () as &mut TelnetChannel<Parent>).on_enable(self.parent, channel),
        }
      },
      Some(ch) => {
        match self.channel_map.get(&ch) {
          Some(&id) => (*self.endpoints.get_mut(id).unwrap()).on_enable(self.parent, channel),
          None => (&mut () as &mut TelnetChannel<Parent>).on_enable(self.parent, channel),
        }
      }
    }
  }
  fn on_disable(&mut self, channel: Option<u8>) {
    match channel {
      None => {
        match self.main {
          Some(ref mut endpoint) => (*endpoint).on_disable(self.parent, channel),
          None => (&mut () as &mut TelnetChannel<Parent>).on_disable(self.parent, channel),
        }
      },
      Some(ch) => {
        match self.channel_map.get(&ch) {
          Some(&id) => (*self.endpoints.get_mut(id).unwrap()).on_disable(self.parent, channel),
          None => (&mut () as &mut TelnetChannel<Parent>).on_disable(self.parent, channel),
        }
      }
    }
  }
  fn on_focus(&mut self, channel: Option<u8>) {
    match channel {
      None => {
        match self.main {
          Some(ref mut endpoint) => (*endpoint).on_focus(self.parent, channel),
          None => (&mut () as &mut TelnetChannel<Parent>).on_focus(self.parent, channel),
        }
      },
      Some(ch) => {
        match self.channel_map.get(&ch) {
          Some(&id) => (*self.endpoints.get_mut(id).unwrap()).on_focus(self.parent, channel),
          None => (&mut () as &mut TelnetChannel<Parent>).on_focus(self.parent, channel),
        }
      }
    }
  }
  fn on_blur(&mut self, channel: Option<u8>) {
    match channel {
      None => {
        match self.main {
          Some(ref mut endpoint) => (*endpoint).on_blur(self.parent, channel),
          None => (&mut () as &mut TelnetChannel<Parent>).on_blur(self.parent, channel),
        }
      },
      Some(ch) => {
        match self.channel_map.get(&ch) {
          Some(&id) => (*self.endpoints.get_mut(id).unwrap()).on_blur(self.parent, channel),
          None => (&mut () as &mut TelnetChannel<Parent>).on_blur(self.parent, channel),
        }
      }
    }
  }
  fn should_enable(&mut self, channel: Option<u8>, attitude: QAttitude) -> bool {
    match channel {
      None => {
        match self.main {
          Some(ref mut endpoint) => (*endpoint).should_enable(self.parent, channel, attitude),
          None => (&mut () as &mut TelnetChannel<Parent>).should_enable(self.parent, channel, attitude),
        }
      },
      Some(ch) => {
        match self.channel_map.get(&ch) {
          Some(&id) => (*self.endpoints.get_mut(id).unwrap()).should_enable(self.parent, channel, attitude),
          None => (&mut () as &mut TelnetChannel<Parent>).should_enable(self.parent, channel, attitude),
        }
      }
    }
  }
}
