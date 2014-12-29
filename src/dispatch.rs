use parser::{TelnetToken};


pub trait DispatchHandler {
  fn on_data<'a>(&mut self, _data: &'a [u8]) {}
  fn on_command(&mut self, _channel: Option<u8>, _command: u8) {}
}
impl DispatchHandler for () {}


pub trait DispatchExt {
  fn dispatch(&mut self, token: TelnetToken);
}

impl<T: DispatchHandler> DispatchExt for T {
  fn dispatch(&mut self, token: TelnetToken) {
    match token {
      TelnetToken::Text(text) => {
        self.on_data(text);
      }

      TelnetToken::Command(command) => {
        self.on_command(None, command);
      }

      TelnetToken::Negotiation{command, channel} => {
        self.on_command(Some(channel), command);
      }
    }
  }
}
