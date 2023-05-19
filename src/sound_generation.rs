use rand::Rng;
use std::f32::consts::PI;

pub fn generate_wav(filename: &str, target_file_size: usize) -> Result<(), hound::Error> {
    let header_size = 44; // WAV header size in bytes
    let target_audio_data_size = target_file_size.saturating_sub(header_size);
    let bytes_per_sample = 4; // 32 bits per sample / 8 bits per byte

    let num_samples = target_audio_data_size / bytes_per_sample;

    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(filename, spec)?;

    let mut rng = rand::thread_rng();

    for _ in 0..num_samples {
        let t = rng.gen_range(0.0..1.0); // Random time value between 0 and 1
        let frequency = rng.gen_range(20.0..2000.0); // Random frequency between 20 Hz and 2000 Hz
                                                     // let sample = (t * frequency * 2.0 * PI).sin();
                                                     // let amplitude = i32::MAX as f32;
        for _ in 0..spec.channels {
            let sample = (t * frequency * 2.0 * PI).sin();
            let amplitude = i32::MAX as f32 * 0.05;
            writer.write_sample((sample * amplitude) as i32)?;
        }
        // writer.write_sample((sample * amplitude) as i32)?;
    }

    writer.finalize()
}
