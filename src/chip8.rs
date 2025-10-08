use std::time::{Duration, Instant};

use crate::{
  audio::Audio,
  keyboard::{KeyCode, KeyState},
};

pub static CYCLE_HZ: usize = 750;
pub static TIMER_HZ: usize = 15;
pub static DISPLAY_HZ: usize = 45;

pub static KEY_SIZE: usize = 16;
pub static STACK_SIZE: usize = 16;
pub static MEMORY_SIZE: usize = 4096;
pub static DISPLAY_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;
pub static DISPLAY_WIDTH: usize = 64;
pub static DISPLAY_HEIGHT: usize = 32;
pub static REGISTERS_SIZE: usize = 16;
pub static ROM_START_ADDRESS: usize = 0x200;
pub static FONTS: [u8; 80] = [
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
pub static KEYBOARD_MAP: [KeyCode; 16] = [
  KeyCode::Key1, // 1
  KeyCode::Key2, // 2
  KeyCode::Key3, // 3
  KeyCode::Key4, // C
  KeyCode::Q,    // 4
  KeyCode::W,    // 5
  KeyCode::E,    // 6
  KeyCode::R,    // D
  KeyCode::A,    // 7
  KeyCode::S,    // 8
  KeyCode::D,    // 9
  KeyCode::F,    // E
  KeyCode::Z,    // A
  KeyCode::X,    // 0
  KeyCode::C,    // B
  KeyCode::V,    // F
];

#[derive(Debug, PartialEq)]
enum Instruction {
  ///00E0
  Clear,
  ///00EE
  Ret,
  ///1NNN
  Jump(u16),
  ///2NNN
  Call(u16),
  ///3XNN
  SkipEqualByte(u8, u8),
  ///4XNN
  SkipNotEqualByte(u8, u8),
  ///5XY0
  SkipEqualRegisters(u8, u8),
  ///6XNN
  LoadByte(u8, u8),
  ///7XNN
  AddRegister(u8, u8),
  ///8XY0
  LoadRegister(u8, u8),
  ///8XY1
  Or(u8, u8),
  ///8XY2
  And(u8, u8),
  ///8XY3
  Xor(u8, u8),
  ///8XY4
  Add(u8, u8),
  ///8XY5
  Subtract(u8, u8),
  ///8XY6
  Shr(u8, u8),
  ///8XY7
  SubtractRev(u8, u8),
  ///8XYE
  Shl(u8, u8),
  ///9XY0
  SkipNotEqualRegisters(u8, u8),
  ///ANNN
  LoadI(u16),
  ///BNNN
  JumpOffset(u8, u16),
  ///CXNN
  Random(u8, u8),
  ///DXYN
  Draw(u8, u8, u8),
  ///EX9E
  SkipKeyPressed(u8),
  ///EXA1
  SkipKeyReleased(u8),
  ///FX07
  LoadDelayTimer(u8),
  ///FX0A
  GetKey(u8),
  ///FX15
  SetDelayTimer(u8),
  ///FX18
  SetSoundTimer(u8),
  ///FX1E
  AddI(u8),
  ///FX29
  LoadFont(u8),
  ///FX33
  LoadBcd(u8),
  ///FX55
  StoreMemory(u8),
  ///FX65
  LoadMemory(u8),
}

impl Instruction {
  // 0000            0000           0000            0000
  // |-instruction-| |-x-register-| |-y-register-|  |-4-bit number-|
  //                                |----8-bit immediate number----|
  //                 |-------12-bit immediate memory address-------|
  fn from(opcode: u16) -> Self {
    let i = ((opcode & 0xF000) >> 12) as u8;
    let x = ((opcode & 0x0F00) >> 8) as u8;
    let y = ((opcode & 0x00F0) >> 4) as u8;
    let n = (opcode & 0x000F) as u8;
    let nn = (opcode & 0x00FF) as u8;
    let nnn = (opcode & 0x0FFF) as u16;

    match i {
      0x0 => match nn {
        0xE0 => Instruction::Clear,
        0xEE => Instruction::Ret,
        _ => panic!("Invalid instruction: {:?}", opcode),
      },
      0x1 => Instruction::Jump(nnn),
      0x2 => Instruction::Call(nnn),
      0x3 => Instruction::SkipEqualByte(x, nn),
      0x4 => Instruction::SkipNotEqualByte(x, nn),
      0x5 => Instruction::SkipEqualRegisters(x, y),
      0x6 => Instruction::LoadByte(x, nn),
      0x7 => Instruction::AddRegister(x, nn),
      0x8 => match n {
        0x0 => Instruction::LoadRegister(x, y),
        0x1 => Instruction::Or(x, y),
        0x2 => Instruction::And(x, y),
        0x3 => Instruction::Xor(x, y),
        0x4 => Instruction::Add(x, y),
        0x5 => Instruction::Subtract(x, y),
        0x6 => Instruction::Shr(x, y),
        0x7 => Instruction::SubtractRev(x, y),
        0xE => Instruction::Shl(x, y),
        _ => panic!("Invalid instruction: {:?}", opcode),
      },
      0x9 => Instruction::SkipNotEqualRegisters(x, y),
      0xA => Instruction::LoadI(nnn),
      0xB => Instruction::JumpOffset(x, nnn),
      0xC => Instruction::Random(x, nn),
      0xD => Instruction::Draw(x, y, n),
      0xE => match nn {
        0x9E => Instruction::SkipKeyPressed(x),
        0xA1 => Instruction::SkipKeyReleased(x),
        _ => panic!("Invalid instruction: {:?}", opcode),
      },
      0xF => match nn {
        0x07 => Instruction::LoadDelayTimer(x),
        0x0A => Instruction::GetKey(x),
        0x15 => Instruction::SetDelayTimer(x),
        0x18 => Instruction::SetSoundTimer(x),
        0x1E => Instruction::AddI(x),
        0x29 => Instruction::LoadFont(x),
        0x33 => Instruction::LoadBcd(x),
        0x55 => Instruction::StoreMemory(x),
        0x65 => Instruction::LoadMemory(x),
        _ => panic!("Invalid instruction: {:?}", opcode),
      },
      _ => panic!("Invalid instruction: {:?}", opcode),
    }
  }
}

#[derive(Debug)]
pub struct Chip8 {
  i: u16,
  pc: u16,
  sp: u16,
  keys: [KeyState; KEY_SIZE],
  stack: [u16; STACK_SIZE],
  memory: [u8; MEMORY_SIZE],
  display: [u8; DISPLAY_SIZE],
  registers: [u8; REGISTERS_SIZE],
  delay_timer: u8,
  sound_timer: u8,

  audio: Audio,
  can_draw: bool,
  shift_quirk: bool,
  cycle_start: Instant,
  timer_start: Instant,
  display_start: Instant,
  cycle_duration: Duration,
  timer_duration: Duration,
  display_duration: Duration,
  current_instruction: Instruction,
}

impl Chip8 {
  pub fn get_display(&self) -> [u8; DISPLAY_SIZE] {
    self.display
  }

  pub fn get_can_draw(&self) -> bool {
    let display_elapsed = self.display_start.elapsed();

    if self.can_draw && display_elapsed >= self.display_duration {
      return true;
    }

    false
  }

  pub fn set_can_draw(&mut self, can_draw: bool) {
    if !can_draw {
      self.display_start = Instant::now();
    }
    self.can_draw = can_draw;
  }
}

impl Chip8 {
  pub fn new(audio: Audio) -> Self {
    let mut chip8 = Self {
      i: 0,
      pc: ROM_START_ADDRESS as u16,
      sp: 0,
      keys: [KeyState::Released; KEY_SIZE],
      stack: [0; STACK_SIZE],
      memory: [0; MEMORY_SIZE],
      display: [0; DISPLAY_SIZE],
      registers: [0; REGISTERS_SIZE],
      delay_timer: 0,
      sound_timer: 0,

      audio,
      can_draw: false,
      shift_quirk: false,
      cycle_start: Instant::now(),
      timer_start: Instant::now(),
      display_start: Instant::now(),
      cycle_duration: Duration::from_micros(1_000_000 / CYCLE_HZ as u64),
      timer_duration: Duration::from_micros(1_000_000 / TIMER_HZ as u64),
      display_duration: Duration::from_micros(1_000_000 / DISPLAY_HZ as u64),
      current_instruction: Instruction::Clear,
    };

    for i in 0..FONTS.len() {
      chip8.memory[i] = FONTS[i];
    }

    chip8
  }

  pub fn sync(&mut self) {
    self.timer_start = Instant::now();
    self.display_start = Instant::now();
  }

  pub fn load_rom(&mut self, rom: &[u8]) {
    for (i, byte) in rom.iter().enumerate() {
      self.memory[ROM_START_ADDRESS + i] = *byte;
    }
  }

  pub fn init_cycle(&mut self) {
    self.cycle_start = Instant::now();
  }

  pub fn cycle(&mut self, key_states: [KeyState; KEY_SIZE]) {
    self.update_keys(key_states);
    self.fetch();
    self.execute();
    self.update_timers();
  }

  pub fn wait_cycle(&mut self) {
    let cycle_elapsed = self.cycle_start.elapsed();
    if cycle_elapsed < self.cycle_duration {
      std::thread::sleep(self.cycle_duration - cycle_elapsed);
    }
  }

  fn fetch(&mut self) {
    let pc = self.pc as usize;
    let instruction_most = self.memory[pc] as u16;
    let instruction_least = self.memory[pc + 1] as u16;
    let opcode = (instruction_most << 8) | instruction_least;
    self.pc += 2;
    self.current_instruction = Instruction::from(opcode);
  }

  fn execute(&mut self) {
    match self.current_instruction {
      Instruction::Clear => self.clear(),
      Instruction::Ret => self.ret(),
      Instruction::Jump(address) => self.jump(address),
      Instruction::Call(address) => self.call(address),
      Instruction::SkipEqualByte(x, nn) => self.skip_equal_byte(x, nn),
      Instruction::SkipNotEqualByte(x, nn) => self.skip_not_equal_byte(x, nn),
      Instruction::SkipEqualRegisters(x, y) => self.skip_equal_registers(x, y),
      Instruction::LoadByte(x, nn) => self.load_byte(x, nn),
      Instruction::AddRegister(x, nn) => self.add_register(x, nn),
      Instruction::LoadRegister(x, y) => self.load_register(x, y),
      Instruction::Or(x, y) => self.or(x, y),
      Instruction::And(x, y) => self.and(x, y),
      Instruction::Xor(x, y) => self.xor(x, y),
      Instruction::Add(x, y) => self.add(x, y),
      Instruction::Subtract(x, y) => self.subtract(x, y),
      Instruction::Shr(x, y) => self.shr(x, y),
      Instruction::SubtractRev(x, y) => self.subtract_rev(x, y),
      Instruction::Shl(x, y) => self.shl(x, y),
      Instruction::SkipNotEqualRegisters(x, y) => self.skip_not_equal_registers(x, y),
      Instruction::LoadI(nnn) => self.load_i(nnn),
      Instruction::JumpOffset(x, nnn) => self.jump_offset(x, nnn),
      Instruction::Random(x, nn) => self.random(x, nn),
      Instruction::Draw(x, y, n) => self.draw(x, y, n),
      Instruction::SkipKeyPressed(x) => self.skip_key_pressed(x),
      Instruction::SkipKeyReleased(x) => self.skip_key_released(x),
      Instruction::LoadDelayTimer(x) => self.load_delay_timer(x),
      Instruction::GetKey(x) => self.get_key(x),
      Instruction::SetDelayTimer(x) => self.set_delay_timer(x),
      Instruction::SetSoundTimer(x) => self.set_sound_timer(x),
      Instruction::AddI(x) => self.add_i(x),
      Instruction::LoadFont(x) => self.load_font(x),
      Instruction::LoadBcd(x) => self.load_bcd(x),
      Instruction::StoreMemory(x) => self.store_memory(x),
      Instruction::LoadMemory(x) => self.load_memory(x),
    }
  }

  fn update_keys(&mut self, key_states: [KeyState; 16]) {
    self.keys[0x1] = key_states[0x0];
    self.keys[0x2] = key_states[0x1];
    self.keys[0x3] = key_states[0x2];
    self.keys[0xC] = key_states[0x3];

    self.keys[0x4] = key_states[0x4];
    self.keys[0x5] = key_states[0x5];
    self.keys[0x6] = key_states[0x6];
    self.keys[0xD] = key_states[0x7];

    self.keys[0x7] = key_states[0x8];
    self.keys[0x8] = key_states[0x9];
    self.keys[0x9] = key_states[0xA];
    self.keys[0xE] = key_states[0xB];

    self.keys[0xA] = key_states[0xC];
    self.keys[0x0] = key_states[0xD];
    self.keys[0xB] = key_states[0xE];
    self.keys[0xF] = key_states[0xF];
  }

  fn update_timers(&mut self) {
    let timer_elapsed = self.timer_start.elapsed();

    if timer_elapsed >= self.timer_duration {
      self.update_delay_timer();
      self.update_sound_timer();
      self.timer_start = Instant::now();
    }
  }

  fn update_delay_timer(&mut self) {
    if self.delay_timer > 0 {
      self.delay_timer -= 1;
    }
  }

  fn update_sound_timer(&mut self) {
    if self.sound_timer > 0 {
      self.sound_timer -= 1;
      self.audio.play(600.0);
    } else {
      self.audio.stop();
    }
  }
}

impl Chip8 {
  fn clear(&mut self) {
    self.display = [0; DISPLAY_SIZE];
  }

  fn ret(&mut self) {
    self.sp -= 1;
    let address = self.stack[self.sp as usize];
    self.jump(address);
  }

  fn jump(&mut self, address: u16) {
    self.pc = address;
  }

  fn call(&mut self, address: u16) {
    self.stack[self.sp as usize] = self.pc;
    self.sp += 1;
    self.jump(address);
  }

  fn skip_equal_byte(&mut self, register_x: u8, value: u8) {
    let register_value = self.registers[register_x as usize];
    if register_value == value {
      self.pc += 2;
    }
  }

  fn skip_not_equal_byte(&mut self, register_x: u8, value: u8) {
    let register_value = self.registers[register_x as usize];
    if register_value != value {
      self.pc += 2;
    }
  }

  fn skip_equal_registers(&mut self, register_x: u8, register_y: u8) {
    let register_x_value = self.registers[register_x as usize];
    let register_y_value = self.registers[register_y as usize];
    if register_x_value == register_y_value {
      self.pc += 2;
    }
  }

  fn load_byte(&mut self, register_x: u8, value: u8) {
    self.registers[register_x as usize] = value;
  }

  fn add_register(&mut self, register_x: u8, value: u8) {
    self.registers[register_x as usize] = self.registers[register_x as usize].wrapping_add(value);
  }

  fn load_register(&mut self, register_x: u8, register_y: u8) {
    self.registers[register_x as usize] = self.registers[register_y as usize];
  }

  fn or(&mut self, register_x: u8, register_y: u8) {
    self.registers[register_x as usize] |= self.registers[register_y as usize];
  }

  fn and(&mut self, register_x: u8, register_y: u8) {
    self.registers[register_x as usize] &= self.registers[register_y as usize];
  }

  fn xor(&mut self, register_x: u8, register_y: u8) {
    self.registers[register_x as usize] ^= self.registers[register_y as usize];
  }

  fn add(&mut self, register_x: u8, register_y: u8) {
    let (sum, carry) =
      self.registers[register_x as usize].overflowing_add(self.registers[register_y as usize]);
    self.registers[register_x as usize] = sum;
    self.registers[0xF] = if carry { 1 } else { 0 };
  }

  fn subtract(&mut self, register_x: u8, register_y: u8) {
    let (result, borrow) =
      self.registers[register_x as usize].overflowing_sub(self.registers[register_y as usize]);
    self.registers[register_x as usize] = result;
    self.registers[0xF] = if borrow { 0 } else { 1 }
  }

  fn shr(&mut self, register_x: u8, register_y: u8) {
    let (x, y) = (register_x as usize, register_y as usize);

    if self.shift_quirk {
      self.registers[x] = self.registers[y];
    }

    self.registers[0xF] = self.registers[x] & 0x1;
    self.registers[x] >>= 1;
  }

  fn subtract_rev(&mut self, register_x: u8, register_y: u8) {
    let (result, borrow) =
      self.registers[register_y as usize].overflowing_sub(self.registers[register_x as usize]);
    self.registers[register_x as usize] = result;
    self.registers[0xF] = if borrow { 0 } else { 1 }
  }

  fn shl(&mut self, register_x: u8, register_y: u8) {
    let (x, y) = (register_x as usize, register_y as usize);

    if self.shift_quirk {
      self.registers[x] = self.registers[y];
    }

    self.registers[0xF] = (self.registers[x] >> 7) & 0x1;
    self.registers[x] <<= 1;
  }

  fn skip_not_equal_registers(&mut self, register_x: u8, register_y: u8) {
    let register_x_value = self.registers[register_x as usize];
    let register_y_value = self.registers[register_y as usize];
    if register_x_value != register_y_value {
      self.pc += 2;
    }
  }

  fn load_i(&mut self, address: u16) {
    self.i = address;
  }

  fn jump_offset(&mut self, register_x: u8, address: u16) {
    self.pc = self.registers[register_x as usize] as u16 + address;
  }

  fn random(&mut self, register_x: u8, value: u8) {
    let random_number = rand::random::<u8>();
    self.registers[register_x as usize] = random_number & value;
  }

  fn draw(&mut self, x: u8, y: u8, n: u8) {
    let x_coord = (self.registers[x as usize] % DISPLAY_WIDTH as u8) as usize;
    let y_coord = (self.registers[y as usize] % DISPLAY_HEIGHT as u8) as usize;

    self.registers[0xF] = 0;

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
            self.registers[0xF] = 1; // Collision detected
          } else {
            self.display[display_offset] = 1;
          }
        }
      }
    }

    self.set_can_draw(true);
  }

  fn skip_key_pressed(&mut self, register_x: u8) {
    let key = self.registers[register_x as usize];
    if self.keys[key as usize] == KeyState::Pressed {
      self.pc += 2;
    }
  }

  fn skip_key_released(&mut self, register_x: u8) {
    let key = self.registers[register_x as usize];
    if self.keys[key as usize] == KeyState::Released {
      self.pc += 2;
    }
  }

  fn load_delay_timer(&mut self, register_x: u8) {
    self.registers[register_x as usize] = self.delay_timer;
  }

  fn get_key(&mut self, register_x: u8) {
    if let Some(key) = self.keys.into_iter().position(|x| x == KeyState::Pressed) {
      self.registers[register_x as usize] = key as u8;
    } else {
      self.pc -= 2;
    }
  }

  fn set_delay_timer(&mut self, register_x: u8) {
    self.delay_timer = self.registers[register_x as usize];
  }

  fn set_sound_timer(&mut self, register_x: u8) {
    self.sound_timer = self.registers[register_x as usize];
  }

  fn add_i(&mut self, register_x: u8) {
    let (result, carry) = self
      .i
      .overflowing_add(self.registers[register_x as usize] as u16);
    self.i = result;
    self.registers[0xF] = if carry { 1 } else { 0 }
  }

  fn load_font(&mut self, register_x: u8) {
    let character = self.registers[register_x as usize];
    self.i = (character * 5) as u16
  }

  fn load_bcd(&mut self, register_x: u8) {
    let register_x_value = self.registers[register_x as usize];
    let first_digit = register_x_value / 100;
    let second_digit = register_x_value % 100 / 10;
    let third_digit = register_x_value % 10;
    self.memory[self.i as usize + 0] = first_digit;
    self.memory[self.i as usize + 1] = second_digit;
    self.memory[self.i as usize + 2] = third_digit;
  }

  fn store_memory(&mut self, register_x: u8) {
    for x in 0..=register_x {
      let data = self.registers[x as usize];
      self.memory[self.i as usize + x as usize] = data;
    }
  }

  fn load_memory(&mut self, register_x: u8) {
    for x in 0..=register_x {
      let data = self.memory[self.i as usize + x as usize];
      self.registers[x as usize] = data;
    }
  }
}
