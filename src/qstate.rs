#![allow(dead_code)]

#[deriving(Show, Copy, PartialEq)]
pub enum QStateUni {
  Disabled,
  AwaitEnable,
  Enabled,
  AwaitDisable,
}

impl QStateUni {
  pub fn is_enabled(self) -> bool {
    self == QStateUni::Enabled || self == QStateUni::AwaitDisable
  }

  pub fn is_disabled(self) -> bool {
    !self.is_enabled()
  }
}

#[deriving(Show, Copy)]
pub struct QState {
  pub local: QStateUni,
  pub remote: QStateUni,
}

impl QState {
  pub fn new() -> QState {
    QState{local: QStateUni::Disabled, remote: QStateUni::Disabled}
  }

  pub fn is_active(&self) -> bool {
    self.local.is_enabled() || self.remote.is_enabled()
  }
}
