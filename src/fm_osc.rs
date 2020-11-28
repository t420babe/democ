use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, GainNode, OscillatorNode, OscillatorType};

/// A midi note is an integer, generally in the range of 21 to 108
pub fn midi_to_freq(note: u8) -> f32 {
  27.5 * 2f32.powf((note as f32 - 21.0) / 12.0)
}

#[wasm_bindgen]
pub struct FmOsc {
  /// Audio context
  ctx: AudioContext,

  /// Primary oscillator (fundamental frequency)
  primary: OscillatorNode,

  /// Overall gain (volume) control
  gain: GainNode,

  /// Amount of frequency modulation
  fm_gain: GainNode,

  /// The oscillator that will modulate the primary oscillator's frequency
  fm_osc: OscillatorNode,

  /// The ratio between the primary frequency and the `fm_osc` frequency
  fm_freq_ratio: f32,

  /// The ratio between the primary frequency and the `fm_gain` frequency
  fm_gain_ratio: f32,
}

impl Drop for FmOsc {
  fn drop(&mut self) {
    let _ = self.ctx.close();
  }
}

#[wasm_bindgen]
impl FmOsc {
  #[wasm_bindgen(constructor)]
  pub fn new() -> Result<FmOsc, JsValue> {
    let audio_context = AudioContext::new()?;

    // Create web audio objects
    let primary = audio_context.create_oscillator()?;
    let fm_osc = audio_context.create_oscillator()?;
    let gain = audio_context.create_gain()?;
    let fm_gain = audio_context.create_gain()?;

    // Initial conditions
    primary.set_type(OscillatorType::Sine);
    primary.frequency().set_value(440.0); // A4 note
    gain.gain().set_value(0.0); // Starts muted
    fm_gain.gain().set_value(0.0); // No initial frequency modulation
    fm_osc.set_type(OscillatorType::Sine);
    fm_osc.frequency().set_value(0.0);

    // Connect the nodes

    // The primary oscillator is routed through the gain node so that it can control the overall
    // output volue
    primary.connect_with_audio_node(&gain)?;

    // Connect the gain node to the `AudioContext` destination (aka the speakers)
    gain.connect_with_audio_node(&audio_context.destination())?;

    // The FM oscillator is connected to its own gain node so it can control the modulation amount
    fm_osc.connect_with_audio_node(&fm_gain)?;

    // Connect the FM oscillator to the frequency parameter of the main oscillator so that the FM
    // node can module its frequency
    fm_gain.connect_with_audio_param(&primary.frequency())?;

    // Start the oscilators
    primary.start()?;
    fm_osc.start()?;

    Ok(FmOsc {
      ctx: audio_context,
      primary,
      gain,
      fm_gain,
      fm_osc,
      fm_freq_ratio: 0.0,
      fm_gain_ratio: 0.0,
    })
  }

  #[wasm_bindgen]
  pub fn set_gain(&self, mut gain: f32) {
    if gain > 1.0 {
      gain = 1.0;
    }
    if gain < 0.0 {
      gain = 0.0;
    }
    self.gain.gain().set_value(gain);
  }

  #[wasm_bindgen]
  pub fn set_primary_frequency(&self, freq: f32) {
    self.primary.frequency().set_value(freq);
    // The frequency of the FM oscillator depends on the frequency of the primary oscillator, so we
    // update the frequency of both in this method
    self.fm_osc.frequency().set_value(self.fm_freq_ratio * freq);
    self.fm_gain.gain().set_value(self.fm_gain_ratio * freq);
  }

  #[wasm_bindgen]
  pub fn set_note(&self, note: u8) {
    let freq = midi_to_freq(note);
    self.set_primary_frequency(freq);
  }

  #[wasm_bindgen]
  pub fn set_fm_amount(&mut self, amount: f32) {
    self.fm_gain_ratio = amount;
    self.fm_gain.gain().set_value(self.fm_gain_ratio * self.primary.frequency().value());
  }

  #[wasm_bindgen]
  pub fn set_fm_frequency(&mut self, amount: f32) {
    self.fm_freq_ratio = amount;
    self.fm_osc.frequency().set_value(self.fm_freq_ratio * self.primary.frequency().value());
  }
}
