mod encryption;
mod sound_generation;
mod soundcloudapi;

use std::error::Error;
use crate::encryption::{
    *
};
use crate::sound_generation::generate_wav;
use crate::soundcloudapi::upload_to_soundcloud;
use clap::Parser;
use klask::Settings;

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

    #[clap(long, default_value = "150000")]
    filesize: usize,
}

#[derive(Parser)]
struct Encode {
    ///Path to the input WAV file
    #[clap(short, long, required = true)]
    input: String,

    ///Path to the output WAV file
    #[clap(short, long, required = true)]
    output: String,

    ///Number of channels
    #[clap(short, long, required = true)]
    message: String,

    #[clap(long)]
    upload: bool,

    #[clap(value_name = "ID", long, requires = "upload")]
    soundcloud_client_id: Option<String>,

    #[clap(value_name = "TOKEN", long, requires = "upload")]
    soundcloud_oauth_token: Option<String>,
}

#[derive(Parser)]
struct Decode {
    ///Path to the input WAV file
    #[clap(short, long)]
    input: String,

    #[clap(short, long)]
    key: String,

    #[clap(long)]
    iv: String,
}
fn main() {
    let mut encryptor = encryption::EncryptDecrypt::new();
    klask::run_derived::<Opts, _>(Settings::default(), |opts| match opts.subcmd {
        SubCommand::Encode(t) => {
            let input_file = t.input;
            let output_file = t.output;
            let message = t.message;

            match &encryptor.encrypt_string_with_stream_cipher(&message) {
                Ok(data) => {
                    match encode_message(&input_file, &output_file, &data) {
                        Ok(_) => {
                            println!("Message encoded successfully!");
                            println!(
                                "key: {}, iv: {}",
                                base64::encode(&encryptor.get_key()),
                                base64::encode(&encryptor.get_iv())
                            );
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
                        }
                        Err(e) => {
                            println!("Error encoding message: {}", e);
                        }
                    }

                }
                Err(e) => eprintln!("Error encoding message: {}", e),
            }
        }
        //Message encoded successfully!
        // key: bgNoyo0PxEBcAg6k+iBhIK9DDh2QE6ndsyCUdY0FJCw=, iv: 8U/ntaiKFgkYjOuL
        SubCommand::Decode(t) => {
            let input_file = t.input;

            match decode_message(
                &input_file,
            ) {
                Ok(decoded_message) => {
                    match EncryptDecrypt::decrypt_string_with_stream_cipher(&decoded_message,                &*base64::decode(&t.key).unwrap(),
                                                                &*base64::decode(&t.iv).unwrap()) {
                        Ok(m) => {
                            println!("Message decoded {}", m);
                        }
                        Err(e) => {
                            println!("Error decoding message: {}", e);
                        }
                    }
                },
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
    });
}

fn encode_message(
    input_file: &str,
    output_file: &str,
    message: &Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = hound::WavReader::open(input_file)?;
    let spec = reader.spec();

    let mut writer = hound::WavWriter::create(output_file, spec)?;

    let mut message_bytes = (message.len() as u32).to_be_bytes().to_vec();

    // let encrypted_message = encrypt_string_with_stream_cipher(message)?;
    message_bytes.extend_from_slice(message);
    let mut bit_index = 0;

    // Calculate available space in input file
    let available_space = (reader.len()) as usize;

    // Calculate length of message in bits
    let message_length_bits = (message.len() * 8)+1;

    // Check if message fits in available space
    if message_length_bits > available_space {
        return Err("Message is too long to fully encode into file".into());
    }

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

// I should probably refactor this to take a DataKeyPair intead of individual params so that it matches the color of the encode_message function
fn decode_message(
    input_file: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
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
            message_length =
                Some(u32::from_be_bytes(message_bytes[0..4].try_into().unwrap()) as usize);
            message_bytes = vec![0u8; message_length.unwrap()];
            bit_index = 0;
        } else if message_length.is_some() && bit_index == message_length.unwrap() * 8 {
            break;
        }

        if i == (reader_len - 1) as usize {
            return Err(Box::new(hound::Error::FormatError(
                "Incomplete message in the input file",
            )));
        }
    }

    Ok(message_bytes)
}

// Import your encode_message and decode_message functions here
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::fs;
//     use std::io::Read;
//
//     fn compare_wav_files(file1: &str, file2: &str) -> bool {
//         let mut file1_contents = Vec::new();
//         let mut file2_contents = Vec::new();
//
//         fs::File::open(file1)
//             .unwrap()
//             .read_to_end(&mut file1_contents)
//             .unwrap();
//         fs::File::open(file2)
//             .unwrap()
//             .read_to_end(&mut file2_contents)
//             .unwrap();
//
//         file1_contents == file2_contents
//     }
//
//     #[test]
//     fn test_encode_decode() -> Result<(), hound::Error> {
//         let input_file = "original.wav";
//         let output_file = "test_output.wav";
//         let message = "               }
//             }
//         }
//     });
// }
//
// fn encode_message(input_file: &str, output_file: &str, message: &str) -> Result<(), hound::Error> {
//     let mut reader = hound::WavReader::open(input_file)?;
//     let spec = reader.spec();
//
//     let mut writer = hound::WavWriter::create(output_file, spec)?;
//
//     let mut message_bytes = (message.len() as u32).to_be_bytes().to_vec();
//     message_bytes.extend_from_slice(message.as_bytes());
//     let mut bit_index = 0;
//
//     for sample_result in reader.samples::<i32>() {
//         let mut sample = sample_result?;
//         if bit_index < message_bytes.len() * 8 {
//             let byte_index = bit_index / 8;
//             let bit_offset = bit_index % 8;
//             let message_bit = (message_bytes[byte_index] >> (7 - bit_offset)) & 1;
//             sample = (sample & !1) | (message_bit as i32);
//             bit_index += 1;
//         }
//         writer.write_sample(sample)?;
//     }
//
//     writer.finalize()?;
//     Ok(())
// }
//
// fn decode_message(input_file: &str) -> Result<String, hound::Error> {
//     let mut reader = hound::WavReader::open(input_file)?;
//     let mut message_bytes = vec![0u8; 4];
//     let mut bit_index = 0;
//     let mut message_length: Option<usize> = None;
//     let reader_len = reader.len();
//     for (i, sample_result) in reader.samples::<i32>().enumerate() {
//         let sample = sample_result?;
//         let message_bit = sample & 1;
//
//         if bit_index < message_bytes.len() * 8 {
//             let byte_index = bit_index / 8;
//             let bit_offset = bit_index % 8;
//             message_bytes[byte_index] |= (message_bit as u8) << (7 - bit_offset);
//             bit_index += 1;
//         }
//
//         if bit_ind";
//
//         // Encode the message
//         let data = encode_message(input_file, output_file, message).unwrap();
//
//         // Decode the message
//         let decoded_message = decode_message(output_file, data.get_key(), data.get_iv()).unwrap();
//         // Check if the decoded message is the same as the original message
//         assert_eq!(decoded_message, message);
//
//         // Clean up the test output file
//         fs::remove_file(output_file).unwrap();
//
//         Ok(())
//     }
//
//     #[test]
//     fn test_file_size() {
//         let target_file_size = 200_000;
//         let filename = "test_random_sine1.wav";
//
//         generate_wav(filename, target_file_size).unwrap();
//         let file_metadata = fs::metadata(filename).unwrap();
//         let file_size = file_metadata.len() as usize;
//
//         assert!(file_size >= target_file_size);
//
//         // Clean up the test file
//         fs::remove_file(filename).unwrap();
//     }
//
//     #[test]
//     fn test_wav_format() {
//         let target_file_size = 200_000;
//         let filename = "test_random_sine.wav";
//
//         generate_wav(filename, target_file_size).unwrap();
//
//         let reader = hound::WavReader::open(filename).unwrap();
//         let spec = reader.spec();
//         assert_eq!(spec.channels, 2);
//         assert_eq!(spec.sample_rate, 44100);
//         assert_eq!(spec.bits_per_sample, 32);
//         assert_eq!(spec.sample_format, hound::SampleFormat::Int);
//
//         // Clean up the test file
//         fs::remove_file(filename).unwrap();
//     }
//
//     #[test]
//     fn test_encrypt_decrypt() {
//         let plaintext = "This is a test";
//         let ciphertext = encrypt_string_with_stream_cipher(plaintext).unwrap();
//
//         assert_eq!(
//             plaintext,
//             decrypt_string_with_stream_cipher(&ciphertext).unwrap()
//         );
//
//         // this is literally an abomination
//         // unsafe {decrypt_string_with_stream_cipher(
//         //     ciphertext.get_data(),
//         //     std::mem::transmute(&(0..32).map(
//         //         |_| random::<u8>()
//         //     ).collect::<Vec<u8>>()),
//         //     std::mem::transmute(&(0..12).map(
//         //         |_| random::<u8>()
//         //     ).collect::<Vec<u8>>())
//         // ).unwrap();}
//     }
// }
