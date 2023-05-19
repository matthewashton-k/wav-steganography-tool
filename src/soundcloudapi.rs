use reqwest::blocking::{multipart};
use std::fs::File;


pub fn upload_to_soundcloud(
    output_file: &str,
    message: &str,
    _client_id: &str,
    oauth_token: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.soundcloud.com/tracks?oauth_token={}",
        oauth_token
    );

    let file = File::open(output_file).map_err(|e| Box::<dyn std::error::Error>::from(e))?;
    let output_file_clone = output_file.to_owned(); // Clone the output_file reference
    let file = multipart::Part::reader(file)
        .file_name(output_file_clone) // Use the cloned reference here
        .mime_str("audio/wav")
        .map_err(|e| Box::<dyn std::error::Error>::from(e))?;

    let form = multipart::Form::new()
        .text("track[title]", message.to_string())
        .part("track[asset_data]", file);

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", format!("OAuth {}", oauth_token))
        .multipart(form)
        .send()
        .map_err(|e| Box::<dyn std::error::Error>::from(e))?;

    let response_json: serde_json::Value = response
        .json()
        .map_err(|e| Box::<dyn std::error::Error>::from(e))?;
    let track_url = response_json["permalink_url"]
        .as_str()
        .ok_or_else(|| {
            Box::<dyn std::error::Error>::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to get track URL",
            ))
        })?
        .to_string();

    Ok(track_url)
}
