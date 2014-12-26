use parser::{TelnetToken};


pub trait DataEndpoint {
  fn on_data<'a>(&mut self, _: &'a [u8]) {}
}
pub trait CommandEndpoint {
  fn on_command(&mut self, _: Option<u8>, _: u8) {}
}
pub trait TelnetDispatchVisitor {
  fn data_handler(&mut self, _scope: &Fn(&mut DataEndpoint));
  fn command_handler(&mut self, _command: u8, _scope: &Fn(&mut CommandEndpoint));
}


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
