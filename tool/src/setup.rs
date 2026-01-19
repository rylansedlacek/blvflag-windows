use std::fs::{OpenOptions, self};
use std::io::{self, Write};
use dirs_next;
use reqwest::header::{AUTHORIZATION, HeaderValue}; // add for API


pub fn ensure_dirs() -> Result<(), Box<dyn std::error::Error>> {
    let home = dirs_next::home_dir().ok_or("No home directory found")?;

    let required_dirs = [
        home.join("blvflag").join("tool").join("history").join("std_history"),
        home.join("blvflag").join("tool").join("history").join("err_history"),
        home.join("blvflag").join("tool").join("key"),
    ];

    for dir in required_dirs {
        fs::create_dir_all(&dir)?;
    }

    Ok(())
}

//TODO should add a fall back!
pub async fn setup_model() -> Result<(), Box<dyn std::error::Error>> {
    ensure_dirs()?; 
    println!("Welcome to BLVFLAG Setup\n");
    println!("Please enter Auth Token:");

    // TODO ask PC is this hardcodedness is ok.
    let endpoint = "http://34.238.139.12:8080/api/meta-key"; 
    let mut auth_token = String::new(); 

    io::stdin().read_line(&mut auth_token)?;
    let auth_token = auth_token.trim();

    let client = reqwest::Client::new();
    // makes the request
    println!("\nFetching API key...\n");
    let res = client.get(endpoint).header(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", auth_token))?).send().await?; 
    if !res.status().is_success() { println!("No key found, check Auth Token."); return Ok(()); }

    let json: serde_json::Value = res.json().await?; // turn to json
    let api_key = json["api_key"]
        .as_str()
        .ok_or("API key not found in response")?; // fall back

        // same as before here:
    let mut key_dir = dirs_next::home_dir().expect("Failed to get home directory"); // grab home dir
    key_dir.push("blvflag/tool/key");
    fs::create_dir_all(&key_dir)?;
    let key_file_path = key_dir.join("api_key");

    let mut file = OpenOptions::new().create(true).write(true).truncate(true).open(&key_file_path)?;
    writeln!(file, "{}", api_key)?; // write it

    println!("Success! API key saved at {:?}", key_file_path);
    Ok(())
}
