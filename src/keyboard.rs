use windows::Win32::UI::Input::KeyboardAndMouse::*;

#[repr(u16)]
#[derive(Copy, Clone)]
pub enum KeyCode {
  Esc = VK_ESCAPE.0,
  Key1 = VK_1.0,
  Key2 = VK_2.0,
  Key3 = VK_3.0,
  Key4 = VK_4.0,
  Q = VK_Q.0,
  W = VK_W.0,
  E = VK_E.0,
  R = VK_R.0,
  A = VK_A.0,
  S = VK_S.0,
  D = VK_D.0,
  F = VK_F.0,
  Z = VK_Z.0,
  X = VK_X.0,
  C = VK_C.0,
  V = VK_V.0,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum KeyState {
  Pressed,
  Released,
}

#[derive(Debug)]
pub struct KeyboardState;

impl KeyboardState {
  pub fn verify_key(key: KeyCode) -> KeyState {
    let key_state = unsafe { GetAsyncKeyState(key as i32) } as i16;
    let is_pressed = key_state & -0x8000i16 != 0;
    match is_pressed {
      true => KeyState::Pressed,
      false => KeyState::Released,
    }
  }

  pub fn verify_keys(keys: [KeyCode; 16]) -> [KeyState; 16] {
    let mut key_code_states = [KeyState::Released; 16];
    for (index, key_code) in keys.into_iter().enumerate() {
      let vk_code = key_code as i32;
      let key_state = unsafe { GetAsyncKeyState(vk_code) } as i16;
      let is_pressed = key_state & -0x8000i16 != 0;
      key_code_states[index] = match is_pressed {
        true => KeyState::Pressed,
        false => KeyState::Released,
      };
    }
    key_code_states
  }
}
