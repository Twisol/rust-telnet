use self::ParseState::*;

#[deriving(Copy, Show)]
pub enum ParseState {
  Neutral,
  Carriage,
  Command,
  Subchannel(u8),
}

#[deriving(Show)]
pub enum TelnetToken<'a> {
  Text(&'a [u8]),
  Command(u8),
  Negotiation{command: u8, channel: u8},
}

#[deriving(Show)]
pub struct TelnetTokenizer<'a> {
  pub state: ParseState,
  pub data: &'a [u8],
}

impl<'a> Iterator<TelnetToken<'a>> for TelnetTokenizer<'a> {
  fn next(&mut self) -> Option<TelnetToken<'a>> {
    while !self.data.is_empty() {
      let (token, state, data) = step_parser(self.state, self.data);
      self.state = state;
      self.data = data;

      if token.is_some() {
        return token;
      }
    }

    return None;
  }
}



pub type ParseResult<'a> = (Option<TelnetToken<'a>>, ParseState, &'a [u8]);

fn is_negotiation(cmd: u8) -> bool {
  250 <= cmd && cmd <= 254
}


fn carriage_state<'a>(data: &'a [u8]) -> ParseResult {
  if data[0] == b'\n' {
    (Some(TelnetToken::Text(b"\r\n")), Neutral, data[1..])
  } else if data[0] == b'\0' {
    (Some(TelnetToken::Text(b"\r")), Neutral, data[1..])
  } else {
    // invalid stream, technically, but still unambiguous
    (Some(TelnetToken::Text(b"\r")), Neutral, data)
  }
}

fn command_state<'a>(data: &'a [u8]) -> ParseResult {
  if is_negotiation(data[0]) {
    (None, Subchannel(data[0]), data[1..])
  } else if data[0] == b'\xFF' {
    (Some(TelnetToken::Text(b"\xFF")), Neutral, data[1..])
  } else {
    (Some(TelnetToken::Command(data[0])), Neutral, data[1..])
  }
}

fn subchannel_state<'a>(data: &'a [u8], command: u8) -> ParseResult {
  (Some(TelnetToken::Negotiation{command: command, channel: data[0]}), Neutral, data[1..])
}

fn neutral_state<'a>(data: &'a [u8]) -> ParseResult {
  let maybe_idx = data.iter().position(|ch: &u8| -> bool {
    *ch == b'\r' || *ch == b'\xFF'
  });

  match maybe_idx {
    Some(idx) => {
      let token = if idx == 0 {
        None
      } else {
        Some(TelnetToken::Text(data[0..idx]))
      };

      let state = if data[idx] == b'\r' {
        Carriage
      } else {
        Command
      };

      (token, state, data[idx+1..])
    }
    None => {
      // This whole data block is text.
      (Some(TelnetToken::Text(data)), Neutral, b"")
    }
  }
}

pub fn step_parser<'a>(state: ParseState, data: &'a [u8]) -> ParseResult {
  if data.is_empty() {
    return (None, state, data)
  }

  match state {
    Neutral => {
      neutral_state(data)
    }
    Carriage => {
      carriage_state(data)
    }
    Command => {
      command_state(data)
    }
    Subchannel(command) => {
      subchannel_state(data, command)
    }
  }
}
