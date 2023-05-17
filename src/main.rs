mod soundcloudapi;
mod sound_generation;

use clap::Parser;
use klask::Settings;
use crate::sound_generation::generate_wav;
use crate::soundcloudapi::upload_to_soundcloud;


#[derive(Parser)]
#[clap(version = "1.0", author = "Thiv Mcthiv")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Encode(Encode),
    Decode(Decode),
    GenerateWav(GenerateWav),
}

#[derive(Parser)]
struct GenerateWav {
    #[clap(short, long, default_value = "default.wav")]
    filename: String,

    #[clap(short, long, default_value = "150000")]
    filesize: usize,
}

#[derive(Parser)]
struct Encode {
    ///Path to the input WAV file
    #[clap(short, long, required = true)]
    input: String,

    ///Path to the output WAV file
    #[clap(short, long,required=true)]
    output: String,

    ///Number of channels
    #[clap(short, long = "message to encode", required=true)]
    message: String,

    #[clap(long= "Upload the encoded track to SoundCloud")]
    upload: bool,

    #[clap(value_name = "ID", long = "SoundCloud Client ID", requires = "upload")]
    soundcloud_client_id: Option<String>,

    #[clap(value_name = "TOKEN", long = "SoundCloud OAuth Token", requires = "upload")]
    soundcloud_oauth_token: Option<String>,
}

#[derive(Parser)]
struct Decode {
    ///Path to the input WAV file
    #[clap(short, long)]
    input: String,
}
fn main() {
    klask::run_derived::<Opts, _>(Settings::default(), |opts| {
        match opts.subcmd {
            SubCommand::Encode(t) => {
                let input_file = t.input;
                let output_file = t.output;
                let message = t.message;

                match encode_message(&input_file, &output_file, &message) {
                    Ok(()) => {
                        println!("Message encoded successfully.");
                        if t.upload {
                            match upload_to_soundcloud(
                                &output_file,
                                &input_file,
                                &t.soundcloud_client_id.unwrap(),
                                &t.soundcloud_oauth_token.unwrap(),
                            ) {
                                Ok(s) => {
                                    println!("Track uploaded successfully to {}.", s);
                                }
                                Err(e) => {
                                    println!("Error uploading track: {}", e);
                                }
                            }
                        }
                    },
                    Err(e) => eprintln!("Error encoding message: {}", e),
                }
            }
            SubCommand::Decode(t) => {
                let input_file = t.input;

                match decode_message(&input_file) {
                    Ok(decoded_message) => println!("Decoded message: {}", decoded_message),
                    Err(e) => eprintln!("Error decoding message: {}", e),
                }
            }
            SubCommand::GenerateWav(t) => {
                let filename = t.filename;
                let filesize = t.filesize;
                match generate_wav(&filename, filesize) {
                    Ok(_) => println!("WAV file generated successfully."),
                    Err(e) => eprintln!("Error generating WAV file: {}", e),
                }
            }
        }
    });
}

fn encode_message(input_file: &str, output_file: &str, message: &str) -> Result<(), hound::Error> {
    let mut reader = hound::WavReader::open(input_file)?;
    let spec = reader.spec();

    let mut writer = hound::WavWriter::create(output_file, spec)?;

    let mut message_bytes = (message.len() as u32).to_be_bytes().to_vec();
    message_bytes.extend_from_slice(message.as_bytes());
    let mut bit_index = 0;

    for sample_result in reader.samples::<i32>() {
        let mut sample = sample_result?;
        if bit_index < message_bytes.len() * 8 {
            let byte_index = bit_index / 8;
            let bit_offset = bit_index % 8;
            let message_bit = (message_bytes[byte_index] >> (7 - bit_offset)) & 1;
            sample = (sample & !1) | (message_bit as i32);
            bit_index += 1;
        }
        writer.write_sample(sample)?;
    }

    writer.finalize()?;
    Ok(())
}

fn decode_message(input_file: &str) -> Result<String, hound::Error> {
    let mut reader = hound::WavReader::open(input_file)?;
    let mut message_bytes = vec![0u8; 4];
    let mut bit_index = 0;
    let mut message_length: Option<usize> = None;
    let reader_len = reader.len();
    for (i, sample_result) in reader.samples::<i32>().enumerate() {
        let sample = sample_result?;
        let message_bit = sample & 1;

        if bit_index < message_bytes.len() * 8 {
            let byte_index = bit_index / 8;
            let bit_offset = bit_index % 8;
            message_bytes[byte_index] |= (message_bit as u8) << (7 - bit_offset);
            bit_index += 1;
        }

        if bit_index == 32 && message_length.is_none() {
            message_length = Some(u32::from_be_bytes(message_bytes[0..4].try_into().unwrap()) as usize);
            message_bytes = vec![0u8; message_length.unwrap()];
            bit_index = 0;
        } else if message_length.is_some() && bit_index == message_length.unwrap() * 8 {
            break;
        }

        if i == (reader_len - 1) as usize {
            return Err(hound::Error::FormatError("Incomplete message in the input file"));
        }
    }

    let message = String::from_utf8(message_bytes).map_err(|_| hound::Error::FormatError("Invalid UTF-8 message"))?;
    Ok(message)
}

// Import your encode_message and decode_message functions here

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Read;
    use super::*;

    fn compare_wav_files(file1: &str, file2: &str) -> bool {
        let mut file1_contents = Vec::new();
        let mut file2_contents = Vec::new();

        fs::File::open(file1).unwrap().read_to_end(&mut file1_contents).unwrap();
        fs::File::open(file2).unwrap().read_to_end(&mut file2_contents).unwrap();

        file1_contents == file2_contents
    }

    #[test]
    fn test_encode_decode() -> Result<(), hound::Error> {
        let input_file = "original.wav";
        let output_file = "test_output.wav";
        let message = "               }
            }
        }
    });
}

fn encode_message(input_file: &str, output_file: &str, message: &str) -> Result<(), hound::Error> {
    let mut reader = hound::WavReader::open(input_file)?;
    let spec = reader.spec();

    let mut writer = hound::WavWriter::create(output_file, spec)?;

    let mut message_bytes = (message.len() as u32).to_be_bytes().to_vec();
    message_bytes.extend_from_slice(message.as_bytes());
    let mut bit_index = 0;

    for sample_result in reader.samples::<i32>() {
        let mut sample = sample_result?;
        if bit_index < message_bytes.len() * 8 {
            let byte_index = bit_index / 8;
            let bit_offset = bit_index % 8;
            let message_bit = (message_bytes[byte_index] >> (7 - bit_offset)) & 1;
            sample = (sample & !1) | (message_bit as i32);
            bit_index += 1;
        }
        writer.write_sample(sample)?;
    }

    writer.finalize()?;
    Ok(())
}

fn decode_message(input_file: &str) -> Result<String, hound::Error> {
    let mut reader = hound::WavReader::open(input_file)?;
    let mut message_bytes = vec![0u8; 4];
    let mut bit_index = 0;
    let mut message_length: Option<usize> = None;
    let reader_len = reader.len();
    for (i, sample_result) in reader.samples::<i32>().enumerate() {
        let sample = sample_result?;
        let message_bit = sample & 1;

        if bit_index < message_bytes.len() * 8 {
            let byte_index = bit_index / 8;
            let bit_offset = bit_index % 8;
            message_bytes[byte_index] |= (message_bit as u8) << (7 - bit_offset);
            bit_index += 1;
        }

        if bit_ind";

        // Encode the message
        encode_message(input_file, output_file, message)?;

        // Decode the message
        let decoded_message = decode_message(output_file)?;
        // Check if the decoded message is the same as the original message
        assert_eq!(decoded_message, message);

        // Clean up the test output file
        fs::remove_file(output_file).unwrap();

        Ok(())
    }


    #[test]
    #[should_panic(expected = "Incomplete message in the input file")]
    fn test_decode_incomplete_message() {
        let input_file = "original.wav";

        // Attempt to decode an incomplete message
        let _ = decode_message(input_file).unwrap();
    }


    #[test]
    fn test_file_size() {
        let target_file_size = 200_000;
        let filename = "test_random_sine1.wav";

        generate_wav(filename, target_file_size).unwrap();
        let file_metadata = fs::metadata(filename).unwrap();
        let file_size = file_metadata.len() as usize;

        assert!(file_size >= target_file_size);

        // Clean up the test file
        fs::remove_file(filename).unwrap();
    }

    #[test]
    fn test_wav_format() {
        let target_file_size = 200_000;
        let filename = "test_random_sine.wav";

        generate_wav(filename, target_file_size).unwrap();

        let reader = hound::WavReader::open(filename).unwrap();
        let spec = reader.spec();
        assert_eq!(spec.channels, 1);
        assert_eq!(spec.sample_rate, 44100);
        assert_eq!(spec.bits_per_sample, 32);
        assert_eq!(spec.sample_format, hound::SampleFormat::Int);

        // Clean up the test file
        fs::remove_file(filename).unwrap();
    }
}
