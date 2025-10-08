use std::{
  error::Error,
  fmt::{Debug, Formatter},
};

use rodio::{OutputStream, OutputStreamBuilder, Sink, Source, source::SineWave};

pub struct Audio {
  sink: Sink,
  #[allow(dead_code)]
  stream_handle: OutputStream,
}

impl Debug for Audio {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Audio")
      .field("sink", &"Sink (not debuggable)")
      .field("stream_handle", &"OutputStream (not debuggable)")
      .finish()
  }
}

// impl Debug for Sink {
//   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//     f.debug_struct("Sink").finish()
//   }
// }

// impl Debug for OutputStream {
//   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//     f.debug_struct("OutputStream").finish()
//   }
// }

impl Audio {
  pub fn new() -> Result<Self, Box<dyn Error>> {
    let stream_handle = OutputStreamBuilder::open_default_stream()?;
    let sink = Sink::connect_new(&stream_handle.mixer());

    Ok(Self {
      sink,
      stream_handle,
    })
  }

  pub fn play(&self, frequency: f32) {
    let source = SineWave::new(frequency);
    self.sink.append(source.clone().repeat_infinite());
    self.sink.play();
  }

  pub fn stop(&self) {
    self.sink.stop();
  }
}
