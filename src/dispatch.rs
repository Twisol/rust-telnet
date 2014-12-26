use parser::{TelnetToken};


pub trait DataEndpoint {
  fn on_data<'a>(&mut self, _: &'a [u8]) {}
}
pub trait TelnetDataVisitor {
  fn data_handler(&mut self, scope: &Fn(&mut DataEndpoint)) {
    scope.call((&mut (),));
  }
}
impl DataEndpoint for () {}
impl TelnetDataVisitor for () {}

pub trait CommandEndpoint {
  fn on_command(&mut self, _: Option<u8>, _: u8) {}
}
pub trait TelnetCommandVisitor {
  fn command_handler(&mut self, _command: u8, scope: &Fn(&mut CommandEndpoint)) {
    scope.call((&mut (),));
  }
}
impl CommandEndpoint for () {}
impl TelnetCommandVisitor for () {}


// Trait alias
pub trait TelnetDispatchVisitor: TelnetDataVisitor + TelnetCommandVisitor {}
impl<T> TelnetDispatchVisitor for T where T: TelnetDataVisitor + TelnetCommandVisitor {}



pub struct TelnetDispatch<'a> {
  pub context: &'a mut (TelnetDispatchVisitor + 'a),
}

impl<'a> TelnetDispatch<'a> {
  pub fn dispatch(&mut self, token: TelnetToken) {
    match token {
      TelnetToken::Text(text) => {
        self.context.data_handler(&|handler| {
          handler.on_data(text);
        });
      }

      TelnetToken::Command(command) => {
        self.context.command_handler(command, &|handler| {
          handler.on_command(None, command);
        });
      }

      TelnetToken::Negotiation{command, channel} => {
        self.context.command_handler(command, &|handler| {
          handler.on_command(Some(channel), command);
        });
      }
    }
  }
}
