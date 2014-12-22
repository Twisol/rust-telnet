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

// How to assign a custom `is_long_command`:
//  tokenizer.is_long_command = (box |&: _: u8| false) as Box<Fn(u8) -> bool>;
pub struct TelnetTokenizer {
  pub state: ParseState,

  // TODO: Once types of unboxed closures can be written,
  //   replace this with a trait Pred: Fn<(u8), bool>.
  pub is_long_command: Box<Fn<(u8,), bool> + 'static>,
}

impl TelnetTokenizer {
  pub fn new() -> TelnetTokenizer {
    TelnetTokenizer {
      state: ParseState::Neutral,
      is_long_command: box |cmd| 250 <= cmd && cmd <= 254,
    }
  }

  pub fn tokenize<'a, 'b>(&'a mut self, data: &'b [u8]) -> TokenStream<'a, 'b> {
    TokenStream {
      state: self,
      data: data,
    }
  }
}


pub struct TokenStream<'a, 'b> {
  pub state: &'a mut TelnetTokenizer,
  pub data: &'b [u8],
}

type ParseResult<'b> = (Option<TelnetToken<'b>>, ParseState, &'b [u8]);

impl<'a, 'b> TokenStream<'a, 'b> {
  fn carriage_state(&self) -> ParseResult<'b> {
    if self.data[0] == b'\n' {
      (Some(TelnetToken::Text(b"\r\n")), Neutral, self.data[1..])
    } else if self.data[0] == b'\0' {
      (Some(TelnetToken::Text(b"\r")), Neutral, self.data[1..])
    } else {
      // invalid stream, technically, but still unambiguous
      (Some(TelnetToken::Text(b"\r")), Neutral, self.data)
    }
  }

  fn command_state(&self) -> ParseResult<'b> {
    if (*self.state.is_long_command).call((self.data[0],)) {
      (None, Subchannel(self.data[0]), self.data[1..])
    } else if self.data[0] == b'\xFF' {
      (Some(TelnetToken::Text(b"\xFF")), Neutral, self.data[1..])
    } else {
      (Some(TelnetToken::Command(self.data[0])), Neutral, self.data[1..])
    }
  }

  fn subchannel_state(&self, command: u8) -> ParseResult<'b> {
    (Some(TelnetToken::Negotiation{command: command, channel: self.data[0]}), Neutral, self.data[1..])
  }

  fn neutral_state(&self) -> ParseResult<'b> {
    let maybe_idx = self.data.iter().position(|ch: &u8| -> bool {
      *ch == b'\r' || *ch == b'\xFF'
    });

    match maybe_idx {
      Some(idx) => {
        let token = if idx == 0 {
          None
        } else {
          Some(TelnetToken::Text(self.data[0..idx]))
        };

        let state = if self.data[idx] == b'\r' {
          Carriage
        } else {
          Command
        };

        (token, state, self.data[idx+1..])
      }
      None => {
        // This whole data block is text.
        (Some(TelnetToken::Text(self.data)), Neutral, b"")
      }
    }
  }

  pub fn step_parser(&mut self) -> Option<TelnetToken<'b>> {
    if self.data.is_empty() {
      return None
    }

    let (token, state, data) = match self.state.state {
      Neutral => {
        self.neutral_state()
      }
      Carriage => {
        self.carriage_state()
      }
      Command => {
        self.command_state()
      }
      Subchannel(command) => {
        self.subchannel_state(command)
      }
    };

    self.state.state = state;
    self.data = data;
    return token;
  }
}

impl<'a, 'b> Iterator<TelnetToken<'b>> for TokenStream<'a, 'b> {
  fn next(&mut self) -> Option<TelnetToken<'b>> {
    while !self.data.is_empty() {
      let token = self.step_parser();
      if token.is_some() {
        return token;
      }
    }

    return None;
  }
}
