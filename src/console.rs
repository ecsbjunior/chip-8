use std::io;

use crossterm::{cursor, style, terminal};

use crate::chip8::{self, Chip8};

pub struct Console<W>
where
  W: io::Write,
{
  w: W,
}

impl<W> Console<W>
where
  W: io::Write,
{
  pub fn new(w: W) -> Self {
    Self { w }
  }

  pub fn init(&mut self) -> Result<(), io::Error> {
    crossterm::queue!(self.w, cursor::Hide)?;
    crossterm::queue!(self.w, terminal::EnterAlternateScreen)?;
    crossterm::queue!(self.w, terminal::Clear(terminal::ClearType::All))?;
    Ok(())
  }

  pub fn finish(&mut self) -> Result<(), io::Error> {
    crossterm::queue!(self.w, cursor::Show)?;
    crossterm::queue!(self.w, terminal::LeaveAlternateScreen)?;
    Ok(())
  }

  pub fn render(&mut self, chip8: &mut Chip8) -> Result<(), io::Error> {
    if !chip8.get_can_draw() {
      return Ok(());
    }

    let display = chip8.get_display();

    crossterm::queue!(self.w, cursor::MoveTo(0, 1))?;

    for y in 0..chip8::DISPLAY_HEIGHT {
      for x in 0..chip8::DISPLAY_WIDTH {
        match display[y * chip8::DISPLAY_WIDTH + x] {
          1 => crossterm::queue!(self.w, style::Print("██"))?,
          _ => crossterm::queue!(self.w, style::Print("  "))?,
        }
      }
      crossterm::queue!(self.w, style::Print("\n"))?;
    }

    chip8.set_can_draw(false);

    Ok(())
  }
}
