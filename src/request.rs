const PREFIX_URL: &str =
    "https://raw.githubusercontent.com/topojson/topojson-client/refs/heads/master";

pub async fn request(filepath: &str) -> Result<String, String> {
    let url = format!("{PREFIX_URL}/{filepath}")
        .parse::<reqwest::Url>()
        .map_err(|e| format!("Cannot parse the URL: {}", e.to_string()))?;
    Ok(reqwest::get(url)
        .await
        .map_err(|e| format!("Cannot send a request: {}", e.to_string()))?
        .text()
        .await
        .map_err(|e| format!("Cannot get the text from the request: {}", e.to_string()))?)
}
