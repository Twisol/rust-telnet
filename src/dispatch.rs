use parser::{TelnetToken};


pub trait DataEndpoint {
  fn on_data<'a>(&mut self, _data: &'a [u8]) {}
}
impl DataEndpoint for () {}

pub trait CommandEndpoint {
  fn on_command(&mut self, _: Option<u8>, _: u8) {}
}
impl CommandEndpoint for () {}

// Trait alias
pub trait DispatchEndpoint: DataEndpoint + CommandEndpoint {}
impl<T> DispatchEndpoint for T where T: DataEndpoint + CommandEndpoint {}


pub fn dispatch<T>(token: TelnetToken, handler: &mut T)
where T: DispatchEndpoint {
  match token {
    TelnetToken::Text(text) => {
      handler.on_data(text);
    }

    TelnetToken::Command(command) => {
      handler.on_command(None, command);
    }

    TelnetToken::Negotiation{command, channel} => {
      handler.on_command(Some(channel), command);
    }
  }
}
