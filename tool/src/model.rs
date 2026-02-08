use std::error::Error;
use std::fs;
use std::path::PathBuf;
use dirs_next;

pub async fn call_llm(prompt: String) -> Result<String, Box<dyn Error>> {
    let api_key_path: PathBuf = dirs_next::home_dir()
        .expect("Could not find home dir")
        .join("blvflag/tool/key/api_key");

    if !api_key_path.exists() {
        return Err("Missing API key file. Run `blvflag setup`.".into());
    }

    let llama_api_key = fs::read_to_string(&api_key_path)?
        .trim()
        .to_string();

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.llama.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", llama_api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "Llama-4-Maverick-17B-128E-Instruct-FP8",
            "messages": [
                { "role": "system", "content": prompt }
            ],
        }))
        .send()
        .await?;

    let json_response: serde_json::Value = response.json().await?;

    let explanation = json_response["completion_message"]["content"]["text"]
        .as_str()
        .unwrap_or("Error communicating with Llama.")
        .to_string();

    Ok(explanation)
}
