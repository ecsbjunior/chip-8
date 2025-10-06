use windows::Win32::UI::Input::KeyboardAndMouse::*;

static NUMBER_OF_KEYS: usize = 17;

static AVAILABLE_KEY_CODES: [u16; NUMBER_OF_KEYS] = [
  VK_1.0,
  VK_2.0,
  VK_3.0,
  VK_4.0,
  VK_Q.0,
  VK_W.0,
  VK_E.0,
  VK_R.0,
  VK_A.0,
  VK_S.0,
  VK_D.0,
  VK_F.0,
  VK_Z.0,
  VK_X.0,
  VK_C.0,
  VK_V.0,
  VK_ESCAPE.0,
];

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum KeyState {
  Pressed,
  Released,
}

impl KeyState {
  pub fn as_u8(self) -> u8 {
    if self == KeyState::Pressed { 1 } else { 0 }
  }
}

#[derive(Debug)]
pub struct KeyboardState {
  pub current_keys: [KeyState; NUMBER_OF_KEYS],
}

impl KeyboardState {
  pub fn new() -> Self {
    Self {
      current_keys: [KeyState::Released; NUMBER_OF_KEYS],
    }
  }

  pub fn poll(&mut self) {
    for (index, key_code) in AVAILABLE_KEY_CODES.iter().enumerate() {
      let key_state = unsafe { GetAsyncKeyState(*key_code as i32) } as i16;
      let is_pressed = key_state & -0x8000i16 != 0;

      self.current_keys[index] = if is_pressed {
        KeyState::Pressed
      } else {
        KeyState::Released
      };
    }
  }
}
