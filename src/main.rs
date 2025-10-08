mod audio;
mod chip8;
mod console;
mod keyboard;

use std::{error::Error, io};

use crate::{audio::Audio, chip8::Chip8, console::Console, keyboard::KeyboardState};

fn main() -> Result<(), Box<dyn Error>> {
  let audio = Audio::new()?;
  let mut chip8 = Chip8::new(audio);
  let mut console = Console::new(io::stdout());

  chip8.load_rom(include_bytes!("../games/breakout.ch8"));

  console.init()?;

  chip8.sync();

  loop {
    chip8.init_cycle();

    let key_states = KeyboardState::verify_keys(chip8::KEYBOARD_MAP);

    if KeyboardState::verify_key(keyboard::KeyCode::Esc) == keyboard::KeyState::Pressed {
      break;
    }

    chip8.cycle(key_states);

    console.render(&mut chip8)?;

    chip8.wait_cycle();
  }

  console.finish()?;

  Ok(())
}
