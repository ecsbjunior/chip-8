struct Chip8 {
  memory: [u8; 4096],
  display: [u8; 64 * 32],
  pc: u16,
  i: u16,
  sp: u16,
  stack: [u16; 16],
  delay_timer: u8,
  sound_timer: u8,
  v: [u8; 16],
  key: [u8; 16],
}

fn main() {
  println!("Hello, world!");
}
