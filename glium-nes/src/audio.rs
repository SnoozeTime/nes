use snafu::{ResultExt, Snafu};
use std::path::Path;
use tracing::info;

#[derive(Snafu, Debug)]
pub enum AudioError {
    #[snafu(display("Error while initiating SDL context = {}", msg))]
    CannotInitSdl2 { msg: String },

    #[snafu(display("Error while getting SDL Audio Subsystem = {}", msg))]
    CannotGetAudioSystem { msg: String },

    #[snafu(display("Cannot open SDL audio queue = {}", msg))]
    CannotOpenQueue { msg: String },

    #[snafu(display("Error while opening Wav Writer"))]
    WavWriterError { source: hound::Error },

    #[snafu(display("Error while recording sample"))]
    CannotRecordSample { source: hound::Error },

    #[snafu(display("lol"))]
    LOL,
}

pub struct AudioSystem {
    _context: sdl2::Sdl,

    /// Queue samples to play on the computer audio
    queue: sdl2::audio::AudioQueue<i16>,

    /// Add samples to save to the wav file.
    wav_writer: Option<hound::WavWriter<std::io::BufWriter<std::fs::File>>>,
}

impl AudioSystem {
    /// Will initialize the audio system as well as the wav recorder.
    pub fn with_recording<P: AsRef<Path>>(recording_name: P) -> Result<Self, AudioError> {
        let mut system = AudioSystem::init()?;
        let specs = hound::WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let path = recording_name.as_ref();
        info!(msg = "Will record Wav", recording_name = %path.display(), specs = ?specs);

        let writer = hound::WavWriter::create(path, specs).context(WavWriterError {})?;

        system.wav_writer = Some(writer);
        Ok(system)
    }

    /// Will initialize the audio system
    pub fn init() -> Result<Self, AudioError> {
        let context = sdl2::init().map_err(|msg| AudioError::CannotInitSdl2 { msg })?;
        let audio_subsystem = context
            .audio()
            .map_err(|msg| AudioError::CannotGetAudioSystem { msg })?;

        let freq: i32 = 44100;
        let samples: u16 = 1024;
        let channels: u8 = 1;
        let desired_specs = sdl2::audio::AudioSpecDesired {
            freq: Some(freq),
            samples: Some(samples),
            channels: Some(channels),
        };

        let queue = audio_subsystem
            .open_queue::<i16, _>(None, &desired_specs)
            .map_err(|msg| AudioError::CannotOpenQueue { msg })?;

        info!(msg = "Created SDL audio queue", freq = %freq, samples = %samples, channels = %channels);

        Ok(Self {
            _context: context,
            queue,
            wav_writer: None,
        })
    }

    /// Start playing.
    pub fn resume(&self) {
        self.queue.resume();
    }

    /// Play (and record) the samples
    pub fn process_samples(&mut self, samples: &[i16]) -> Result<(), AudioError> {
        self.queue.queue(&samples);
        if let Some(ref mut writer) = self.wav_writer.as_mut() {
            for sample in samples {
                writer
                    .write_sample(*sample)
                    .context(CannotRecordSample {})?;
            }
        }

        Ok(())
    }
}
