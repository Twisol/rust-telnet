use std::collections::{HashMap};
use std::vec::{Vec};
pub use demux::{ChannelHandler};
use qstate::{QAttitude};

pub trait SomeThing<Parent> {
  fn visit(&mut self, parent: &mut Parent, scope: &Fn(&mut ChannelHandler));
  fn ask(&mut self, parent: &mut Parent, scope: &Fn(&mut ChannelHandler) -> bool) -> bool;
}
impl<Parent: ChannelHandler> SomeThing<Parent> for () {
  fn visit(&mut self, parent: &mut Parent, scope: &Fn(&mut ChannelHandler)) {
    scope.call((parent,))
  }
  fn ask(&mut self, parent: &mut Parent, scope: &Fn(&mut ChannelHandler) -> bool) -> bool {
    scope.call((parent,))
  }
}

pub struct EndpointRegistry<'parent, Parent: 'parent> {
  pub parent: &'parent mut Parent,

  pub command_map: HashMap<u8, uint>,
  pub channel_map: HashMap<u8, uint>,
  pub endpoints: Vec<&'parent mut (SomeThing<Parent> + 'parent)>,

  pub main: Option<&'parent mut (SomeThing<Parent> + 'parent)>,
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

  fn _get_command_handler(&mut self, command: u8, scope: &Fn(&mut ChannelHandler)) {
    match self.command_map.get(&command) {
      Some(&id) => self.endpoints.get_mut(id).unwrap().visit(self.parent, scope),
      None => (&mut ()).visit(self.parent, scope),
    }
  }

  fn _get_channel_handler(&mut self, channel: Option<u8>, scope: &Fn(&mut ChannelHandler)) {
    match channel {
      None => {
        match self.main {
          Some(ref mut endpoint) => endpoint.visit(self.parent, scope),
          None => (&mut ()).visit(self.parent, scope),
        }
      },
      Some(ch) => {
        match self.channel_map.get(&ch) {
          Some(&id) => self.endpoints.get_mut(id).unwrap().visit(self.parent, scope),
          None => (&mut ()).visit(self.parent, scope),
        }
      }
    }
  }

  fn _ask_channel_handler(&mut self, channel: Option<u8>, scope: &Fn(&mut ChannelHandler) -> bool) -> bool {
    match channel {
      None => {
        match self.main {
          Some(ref mut endpoint) => endpoint.ask(self.parent, scope),
          None => (&mut ()).ask(self.parent, scope),
        }
      },
      Some(ch) => {
        match self.channel_map.get(&ch) {
          Some(&id) => self.endpoints.get_mut(id).unwrap().ask(self.parent, scope),
          None => (&mut ()).ask(self.parent, scope),
        }
      }
    }
  }
}

impl<'parent, Parent> ChannelHandler for EndpointRegistry<'parent, Parent>
where Parent: ChannelHandler {
  fn on_data<'a>(&mut self, channel: Option<u8>, data: &'a [u8]) {
    self._get_channel_handler(channel, &|handler| {
      handler.on_data(channel, data)
    })
  }
  fn on_command(&mut self, channel: Option<u8>, command: u8) {
    self._get_command_handler(command, &|handler| {
      handler.on_command(channel, command)
    })
  }

  fn on_enable(&mut self, channel: Option<u8>) {
    self._get_channel_handler(channel, &|handler| {
      handler.on_enable(channel)
    })
  }
  fn on_disable(&mut self, channel: Option<u8>) {
    self._get_channel_handler(channel, &|handler|{
      handler.on_disable(channel)
    })
  }
  fn on_focus(&mut self, channel: Option<u8>) {
    self._get_channel_handler(channel, &|handler| {
      handler.on_focus(channel)
    })
  }
  fn on_blur(&mut self, channel: Option<u8>) {
    self._get_channel_handler(channel, &|handler| {
      handler.on_blur(channel)
    })
  }
  fn should_enable(&mut self, channel: Option<u8>, attitude: QAttitude) -> bool {
    self._ask_channel_handler(channel, &|handler| {
      handler.should_enable(channel, attitude)
    })
  }
}
