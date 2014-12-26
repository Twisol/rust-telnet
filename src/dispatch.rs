use parser::{TelnetToken};


pub trait DataEndpoint {
  fn on_data<'a>(&mut self, _: &'a [u8]) {}
}
pub trait CommandEndpoint {
  fn on_command(&mut self, _: Option<u8>, _: u8) {}
}
pub trait TelnetDispatchVisitor {
  fn data_handler<'a>(&'a mut self) -> &'a mut DataEndpoint;
  fn command_handler<'a>(&'a mut self, _command: u8) -> &'a mut CommandEndpoint;
}


pub struct TelnetDispatch<'a> {
  pub context: &'a mut (TelnetDispatchVisitor + 'a),
}

impl<'a> TelnetDispatch<'a> {
  pub fn dispatch(&mut self, token: TelnetToken) {
    match token {
      TelnetToken::Text(text) => {
        self.context.data_handler().on_data(text);
      }

      TelnetToken::Command(command) => {
        self.context.command_handler(command).on_command(None, command);
      }

      TelnetToken::Negotiation{command, channel} => {
        self.context.command_handler(command).on_command(Some(channel), command);
      }
    }
  }
}
