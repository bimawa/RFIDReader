use esp_hal::{i2s::master::I2sTx, Blocking};

const SAMPLE_RATE: u32 = 16000;
const BEEP_FREQ: u32 = 2500;
const BEEP_DURATION_MS: u32 = 150;
const SAMPLES_PER_BEEP: usize = (SAMPLE_RATE * BEEP_DURATION_MS / 1000) as usize;
const SAMPLES_PER_HALF_WAVE: usize = (SAMPLE_RATE / BEEP_FREQ / 2) as usize;

pub struct Audio<'d> {
    i2s_tx: I2sTx<'d, Blocking>,
}

impl<'d> Audio<'d> {
    pub fn new(i2s_tx: I2sTx<'d, Blocking>) -> Self {
        Self { i2s_tx }
    }

    pub fn beep(&mut self) {
        let amplitude: i16 = 32700;
        let mut samples = [0i16; SAMPLES_PER_BEEP * 2];

        for i in 0..SAMPLES_PER_BEEP {
            let sample = if (i / SAMPLES_PER_HALF_WAVE) % 2 == 0 {
                amplitude
            } else {
                -amplitude
            };
            samples[i * 2] = sample;
            samples[i * 2 + 1] = sample;
        }

        let _ = self.i2s_tx.write_words(&samples);
    }
}
