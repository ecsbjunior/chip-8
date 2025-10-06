mod keyboard;

use std::{
  error::Error,
  io::{self},
};

use crossterm::{
  cursor, style,
  terminal::{self},
};

use crate::keyboard::KeyState;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const MEMORY_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
const REGISTERS_SIZE: usize = 16;
const KEY_SIZE: usize = 16;
const FONTS: [u8; 80] = [
  0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
  0x20, 0x60, 0x20, 0x20, 0x70, // 1
  0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
  0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
  0x90, 0x90, 0xF0, 0x10, 0x10, // 4
  0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
  0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
  0xF0, 0x10, 0x20, 0x40, 0x40, // 7
  0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
  0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
  0xF0, 0x90, 0xF0, 0x90, 0x90, // A
  0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
  0xF0, 0x80, 0x80, 0x80, 0xF0, // C
  0xE0, 0x90, 0x90, 0x90, 0xE0, // D
  0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
  0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[derive(Debug, PartialEq)]
enum Instruction {
  Special = 0x0,
  Jump = 0x1,
  Subroutine = 0x2,
  SkipRegisterVEqualNN = 0x3,
  SkipRegisterVNotEqualNN = 0x4,
  SkipRegistersVEqual = 0x5,
  SetRegisterV = 0x6,
  AddRegisterV = 0x7,
  Arithmetic = 0x8,
  SkipRegistersVNotEqual = 0x9,
  SetRegisterI = 0xA,
  JumpOffset = 0xB,
  Random = 0xC,
  Draw = 0xD,
  SkipIfKey = 0xE,
  Others = 0xF,
}

impl Instruction {
  fn from(instruction: u8) -> Self {
    match instruction {
      0x0 => Instruction::Special,
      0x1 => Instruction::Jump,
      0x2 => Instruction::Subroutine,
      0x3 => Instruction::SkipRegisterVEqualNN,
      0x4 => Instruction::SkipRegisterVNotEqualNN,
      0x5 => Instruction::SkipRegistersVEqual,
      0x6 => Instruction::SetRegisterV,
      0x7 => Instruction::AddRegisterV,
      0x8 => Instruction::Arithmetic,
      0x9 => Instruction::SkipRegistersVNotEqual,
      0xA => Instruction::SetRegisterI,
      0xB => Instruction::JumpOffset,
      0xC => Instruction::Random,
      0xD => Instruction::Draw,
      0xE => Instruction::SkipIfKey,
      0xF => Instruction::Others,
      _ => panic!("Invalid instruction: {:?}", instruction),
    }
  }
}

#[derive(Debug)]
struct Opcode {
  instruction: Instruction,
  x: u8,
  y: u8,
  n: u8,
  nn: u8,
  nnn: u16,
}

impl Opcode {
  fn from(opcode: u16) -> Self {
    // 0000            0000           0000            0000
    //|-instruction-| |-x-register-| |-y-register-|  |-4-bit number-|
    //                               |----8-bit immediate number----|
    //                |-------12-bit immediate memory address-------|
    Self {
      instruction: Instruction::from(((opcode & 0xF000) >> 12) as u8),
      x: ((opcode & 0x0F00) >> 8) as u8,
      y: ((opcode & 0x00F0) >> 4) as u8,
      n: ((opcode & 0x000F) as u8),
      nn: ((opcode & 0x00FF) as u8),
      nnn: ((opcode & 0x0FFF) as u16),
    }
  }
}

#[derive(Debug)]
struct Chip8 {
  memory: [u8; MEMORY_SIZE],
  display: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT],
  pc: u16,
  i: u16,
  sp: u16,
  stack: [u16; STACK_SIZE],
  delay_timer: u8,
  sound_timer: u8,
  v: [u8; REGISTERS_SIZE],
  draw: bool,
  keys: [u8; KEY_SIZE],
  shift_quirk: bool,
}

impl Chip8 {
  fn new() -> Self {
    let mut chip8 = Self {
      memory: [0; MEMORY_SIZE],
      display: [0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
      pc: 0x200,
      i: 0,
      sp: 0,
      stack: [0; STACK_SIZE],
      delay_timer: 0,
      sound_timer: 0,
      v: [0; REGISTERS_SIZE],
      draw: false,
      keys: [0; KEY_SIZE],
      shift_quirk: false,
    };

    for i in 0..FONTS.len() {
      chip8.memory[i] = FONTS[i];
    }

    chip8
  }

  fn load_rom(&mut self, rom: &[u8]) {
    for (i, byte) in rom.iter().enumerate() {
      self.memory[i + 0x200] = *byte;
    }
  }

  fn cycle(&mut self) {
    let opcode = self.fetch();
    self.execute(opcode);
  }

  fn update_delay_timer(&mut self) {
    if self.delay_timer > 0 {
      self.delay_timer -= 1;
    }
  }

  fn update_sound_timer(&mut self) {
    if self.sound_timer > 0 {
      self.sound_timer -= 1;
    }
  }

  fn fetch(&mut self) -> Opcode {
    let pc = self.pc as usize;
    let opcode = (self.memory[pc] as u16) << 8 | self.memory[pc + 1] as u16;
    self.pc += 2;
    Opcode::from(opcode)
  }

  fn execute(&mut self, opcode: Opcode) {
    match opcode.instruction {
      Instruction::Special => {
        if opcode.nn == 0xE0 {
          self.clear()
        } else if opcode.nn == 0xEE {
          self.pop_subroutine();
        } else {
          // Execute machine language routine (skip this one)
        }
      }
      Instruction::Jump => self.jump(opcode.nnn),
      Instruction::Subroutine => self.subroutine(opcode.nnn),
      Instruction::SkipRegisterVEqualNN => self.skip_register_v_equal_nn(opcode.x, opcode.nn),
      Instruction::SkipRegisterVNotEqualNN => {
        self.skip_register_v_not_equal_nn(opcode.x, opcode.nn)
      }
      Instruction::SkipRegistersVEqual => self.skip_registers_v_equal(opcode.x, opcode.y),
      Instruction::SetRegisterV => self.set_register_v(opcode.x, opcode.nn),
      Instruction::AddRegisterV => self.add_register_v(opcode.x, opcode.nn),
      Instruction::Arithmetic => {
        if opcode.n == 0x0 {
          self.set_vx_with_vy(opcode.x, opcode.y);
        } else if opcode.n == 0x1 {
          self.bitwise_or(opcode.x, opcode.y);
        } else if opcode.n == 0x2 {
          self.bitwise_and(opcode.x, opcode.y);
        } else if opcode.n == 0x3 {
          self.bitwise_xor(opcode.x, opcode.y);
        } else if opcode.n == 0x4 {
          self.arithmetic_add(opcode.x, opcode.y);
        } else if opcode.n == 0x5 {
          self.arithmetic_subtract(opcode.x, opcode.y);
        } else if opcode.n == 0x6 {
          self.bitwise_shift_right(opcode.x, opcode.y);
        } else if opcode.n == 0x7 {
          self.arithmetic_subtract_rev(opcode.x, opcode.y);
        } else if opcode.n == 0xE {
          self.bitwise_shift_left(opcode.x, opcode.y);
        }
      }
      Instruction::SkipRegistersVNotEqual => self.skip_registers_v_not_equal(opcode.x, opcode.y),
      Instruction::SetRegisterI => self.set_register_i(opcode.nnn),
      Instruction::JumpOffset => self.jump_offset(opcode.x, opcode.nnn),
      Instruction::Random => self.random(opcode.x, opcode.nn),
      Instruction::Draw => self.draw(opcode.x, opcode.y, opcode.n),
      Instruction::SkipIfKey => {
        if opcode.nn == 0x9E {
          self.skip_if_key_pressed(opcode.x)
        } else if opcode.nn == 0xA1 {
          self.skip_if_key_not_pressed(opcode.x)
        }
      }
      Instruction::Others => {
        if opcode.nn == 0x07 {
          self.set_register_v_with_delay_timer(opcode.x);
        } else if opcode.nn == 0x0A {
          self.get_key(opcode.x);
        } else if opcode.nn == 0x15 {
          self.set_delay_timer_with_register_v(opcode.x);
        } else if opcode.nn == 0x18 {
          self.set_sound_timer_with_register_v(opcode.x);
        } else if opcode.nn == 0x29 {
          self.set_register_i_to_font(opcode.x);
        } else if opcode.nn == 0x33 {
          self.binary_coded_decimal_conversion(opcode.x)
        } else if opcode.nn == 0x55 {
          self.store_memory(opcode.x)
        } else if opcode.nn == 0x65 {
          self.load_memory(opcode.x)
        } else if opcode.nn == 0x1E {
          self.add_register_i(opcode.x)
        }
      }
    }
  }

  fn clear(&mut self) {
    self.display = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
  }

  fn jump(&mut self, address: u16) {
    self.pc = address;
  }

  fn subroutine(&mut self, address: u16) {
    self.stack[self.sp as usize] = self.pc;
    self.sp += 1;
    self.jump(address);
  }

  fn pop_subroutine(&mut self) {
    self.sp -= 1;
    let address = self.stack[self.sp as usize];
    self.jump(address);
  }

  fn skip_register_v_equal_nn(&mut self, register_x: u8, value: u8) {
    let register_value = self.v[register_x as usize];
    if register_value == value {
      self.pc += 2;
    }
  }

  fn skip_register_v_not_equal_nn(&mut self, register_x: u8, value: u8) {
    let register_value = self.v[register_x as usize];
    if register_value != value {
      self.pc += 2;
    }
  }

  fn skip_registers_v_equal(&mut self, register_x: u8, register_y: u8) {
    let register_x_value = self.v[register_x as usize];
    let register_y_value = self.v[register_y as usize];
    if register_x_value == register_y_value {
      self.pc += 2;
    }
  }

  fn set_register_v(&mut self, register_x: u8, value: u8) {
    self.v[register_x as usize] = value;
  }

  fn add_register_v(&mut self, register_x: u8, value: u8) {
    self.v[register_x as usize] = self.v[register_x as usize].wrapping_add(value);
  }

  fn skip_registers_v_not_equal(&mut self, register_x: u8, register_y: u8) {
    let register_x_value = self.v[register_x as usize];
    let register_y_value = self.v[register_y as usize];
    if register_x_value != register_y_value {
      self.pc += 2;
    }
  }

  fn set_vx_with_vy(&mut self, register_x: u8, register_y: u8) {
    self.v[register_x as usize] = self.v[register_y as usize];
  }

  fn bitwise_or(&mut self, register_x: u8, register_y: u8) {
    self.v[register_x as usize] |= self.v[register_y as usize];
  }

  fn bitwise_and(&mut self, register_x: u8, register_y: u8) {
    self.v[register_x as usize] &= self.v[register_y as usize];
  }

  fn bitwise_xor(&mut self, register_x: u8, register_y: u8) {
    self.v[register_x as usize] ^= self.v[register_y as usize];
  }

  fn arithmetic_add(&mut self, register_x: u8, register_y: u8) {
    let (sum, carry) = self.v[register_x as usize].overflowing_add(self.v[register_y as usize]);
    self.v[register_x as usize] = sum;
    self.v[0xF] = if carry { 1 } else { 0 };
  }

  fn arithmetic_subtract(&mut self, register_x: u8, register_y: u8) {
    let (result, borrow) = self.v[register_x as usize].overflowing_sub(self.v[register_y as usize]);
    self.v[register_x as usize] = result;
    self.v[0xF] = if borrow { 0 } else { 1 }
  }

  fn arithmetic_subtract_rev(&mut self, register_x: u8, register_y: u8) {
    let (result, borrow) = self.v[register_y as usize].overflowing_sub(self.v[register_x as usize]);
    self.v[register_x as usize] = result;
    self.v[0xF] = if borrow { 0 } else { 1 }
  }

  fn bitwise_shift_right(&mut self, register_x: u8, register_y: u8) {
    let (x, y) = (register_x as usize, register_y as usize);

    if self.shift_quirk {
      self.v[x] = self.v[y];
    }

    self.v[0xF] = self.v[x] & 0x1;
    self.v[x] >>= 1;
  }

  fn bitwise_shift_left(&mut self, register_x: u8, register_y: u8) {
    let (x, y) = (register_x as usize, register_y as usize);

    if self.shift_quirk {
      self.v[x] = self.v[y];
    }

    self.v[0xF] = (self.v[x] >> 7) & 0x1;
    self.v[x] <<= 1;
  }

  fn set_register_i(&mut self, address: u16) {
    self.i = address;
  }

  fn jump_offset(&mut self, register_x: u8, address: u16) {
    self.pc = self.v[register_x as usize] as u16 + address;
  }

  fn random(&mut self, register_x: u8, value: u8) {
    let random_number = rand::random::<u8>();
    self.v[register_x as usize] = random_number & value;
  }

  fn draw(&mut self, x: u8, y: u8, n: u8) {
    let x_coord = (self.v[x as usize] % DISPLAY_WIDTH as u8) as usize;
    let y_coord = (self.v[y as usize] % DISPLAY_HEIGHT as u8) as usize;

    self.v[0xF] = 0;

    for sprite_y in 0..n {
      let target_y = y_coord + sprite_y as usize;

      if target_y >= DISPLAY_HEIGHT {
        break;
      }

      let y_offset = target_y * DISPLAY_WIDTH;
      let sprite_pixels = self.memory[(self.i + sprite_y as u16) as usize];

      for sprite_x in 0..8 {
        let target_x = x_coord + sprite_x;

        if target_x >= DISPLAY_WIDTH {
          break;
        }

        let sprite_pixel = (sprite_pixels >> (7 - sprite_x)) & 1;
        let display_offset = y_offset + target_x;
        let display_pixel = self.display[display_offset];

        if sprite_pixel == 1 {
          if display_pixel == 1 {
            self.display[display_offset] = 0;
            self.v[0xF] = 1; // Collision detected
          } else {
            self.display[display_offset] = 1;
          }
        }
      }
    }

    self.draw = true;
  }

  fn skip_if_key_pressed(&mut self, register_x: u8) {
    let key = self.v[register_x as usize];
    if self.keys[key as usize] & 1 == 1 {
      self.pc += 2;
    }
  }

  fn skip_if_key_not_pressed(&mut self, register_x: u8) {
    let key = self.v[register_x as usize];
    if self.keys[key as usize] & 1 == 0 {
      self.pc += 2;
    }
  }

  fn set_register_v_with_delay_timer(&mut self, register_x: u8) {
    self.v[register_x as usize] = self.delay_timer;
  }

  fn get_key(&mut self, register_x: u8) {
    if let Some(key) = self.keys.into_iter().position(|x| x == 1) {
      self.v[register_x as usize] = key as u8;
    } else {
      self.pc -= 2;
    }
  }

  fn set_delay_timer_with_register_v(&mut self, register_x: u8) {
    self.delay_timer = self.v[register_x as usize];
  }

  fn set_sound_timer_with_register_v(&mut self, register_x: u8) {
    self.sound_timer = self.v[register_x as usize];
  }

  fn set_register_i_to_font(&mut self, register_x: u8) {
    let character = self.v[register_x as usize];
    self.i = (character * 5) as u16
  }

  fn binary_coded_decimal_conversion(&mut self, register_x: u8) {
    let register_x_value = self.v[register_x as usize];
    let first_digit = register_x_value / 100;
    let second_digit = register_x_value % 100 / 10;
    let third_digit = register_x_value % 10;
    self.memory[self.i as usize + 0] = first_digit;
    self.memory[self.i as usize + 1] = second_digit;
    self.memory[self.i as usize + 2] = third_digit;
  }

  fn store_memory(&mut self, register_x: u8) {
    for x in 0..=register_x {
      let data = self.v[x as usize];
      self.memory[self.i as usize + x as usize] = data;
    }
  }

  fn load_memory(&mut self, register_x: u8) {
    for x in 0..=register_x {
      let data = self.memory[self.i as usize + x as usize];
      self.v[x as usize] = data;
    }
  }

  fn add_register_i(&mut self, register_x: u8) {
    let (result, carry) = self.i.overflowing_add(self.v[register_x as usize] as u16);
    self.i = result;
    self.v[0xF] = if carry { 1 } else { 0 }
  }
}

fn main() -> Result<(), Box<dyn Error>> {
  let mut chip8 = Chip8::new();
  let mut stdout = io::stdout();
  let mut keyboard = keyboard::KeyboardState::new();

  chip8.load_rom(include_bytes!("../games/breakout.ch8"));

  crossterm::queue!(stdout, cursor::Hide)?;
  crossterm::queue!(stdout, terminal::EnterAlternateScreen)?;
  crossterm::queue!(stdout, terminal::Clear(terminal::ClearType::All))?;

  let display_hz = 30;
  let display_duration = std::time::Duration::from_micros(1_000_000 / display_hz);

  let cycle_hz = 500;
  let cycle_duration = std::time::Duration::from_micros(1_000_000 / cycle_hz);

  let timer_hz = 60;
  let timer_duration = std::time::Duration::from_micros(1_000_000 / timer_hz);

  let mut display_start = std::time::Instant::now();
  let mut timer_start = std::time::Instant::now();

  loop {
    let cycle_start = std::time::Instant::now();

    keyboard.poll();

    chip8.keys[0x1] = keyboard.current_keys[0x0].as_u8();
    chip8.keys[0x2] = keyboard.current_keys[0x1].as_u8();
    chip8.keys[0x3] = keyboard.current_keys[0x2].as_u8();
    chip8.keys[0xC] = keyboard.current_keys[0x3].as_u8();

    chip8.keys[0x4] = keyboard.current_keys[0x4].as_u8();
    chip8.keys[0x5] = keyboard.current_keys[0x5].as_u8();
    chip8.keys[0x6] = keyboard.current_keys[0x6].as_u8();
    chip8.keys[0xD] = keyboard.current_keys[0x7].as_u8();

    chip8.keys[0x7] = keyboard.current_keys[0x8].as_u8();
    chip8.keys[0x8] = keyboard.current_keys[0x9].as_u8();
    chip8.keys[0x9] = keyboard.current_keys[0xA].as_u8();
    chip8.keys[0xE] = keyboard.current_keys[0xB].as_u8();

    chip8.keys[0xA] = keyboard.current_keys[0xC].as_u8();
    chip8.keys[0x0] = keyboard.current_keys[0xD].as_u8();
    chip8.keys[0xB] = keyboard.current_keys[0xE].as_u8();
    chip8.keys[0xF] = keyboard.current_keys[0xF].as_u8();

    if keyboard.current_keys[0x10] == KeyState::Pressed {
      break;
    }

    chip8.cycle();

    // Atualizar timers a 60 Hz
    let timer_elapsed = timer_start.elapsed();
    if timer_elapsed >= timer_duration {
      chip8.update_delay_timer();
      chip8.update_sound_timer();
      timer_start = std::time::Instant::now();
    }

    // Atualizar display a 30 Hz quando houver mudanças
    if chip8.draw {
      let display_elapsed = display_start.elapsed();

      if display_elapsed >= display_duration {
        crossterm::queue!(stdout, cursor::MoveTo(0, 1))?;

        for y in 0..DISPLAY_HEIGHT {
          for x in 0..DISPLAY_WIDTH {
            match chip8.display[y * DISPLAY_WIDTH + x] {
              1 => crossterm::queue!(stdout, style::Print("██"))?,
              _ => crossterm::queue!(stdout, style::Print("  "))?,
            }
          }
          crossterm::queue!(stdout, style::Print("\n"))?;
        }

        chip8.draw = false;
        display_start = std::time::Instant::now();
      }
    }

    let cycle_elapsed = cycle_start.elapsed();

    if cycle_elapsed < cycle_duration {
      std::thread::sleep(cycle_duration - cycle_elapsed);
    }
  }

  crossterm::queue!(stdout, cursor::Show)?;
  crossterm::queue!(stdout, terminal::LeaveAlternateScreen)?;

  Ok(())
}
