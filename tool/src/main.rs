mod generate;
mod commands;
mod setup;
mod diff;
mod buckets;
mod model;

use clap::{App, Arg, SubCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("blvflag")
        .usage("blvflag [scriptName.py] [--flag]")
        .arg(Arg::new("script")  
            .help("The script to run.")
            .required(false)
            .index(1))
        .arg(Arg::new("explain") 
            .long("explain")      
            .help("Utlized to explain error messages in more verbose manner.")
            .takes_value(false))  
        .arg(Arg::new("context")
            .long("context")
            .help("Utilized to explain error messages using stored development cycles for added context.")
            .takes_value(false))  
        .arg(Arg::new("diff")
            .long("diff")
            .help("Utilized to compare code changes for debugging.")
            .takes_value(false))  
        .arg(Arg::new("revert")
            .long("revert")
            .help("Revert the script back to the previous saved version.")
            .takes_value(false))
        .subcommand(SubCommand::with_name("setup")
            .help("Run this command to link your key to the Llama API.")
            .about("Prompts for API key to use Llama."))
        .subcommand(SubCommand::with_name("clear")
            .help("If directories don't exist, try running a program, or re-install.")
            .about("Clears history directories except for placeholder.json files."))
        .get_matches();

        if let Some(script) = matches.value_of("script") { 
            let explain = matches.is_present("explain");
            let diff = matches.is_present("diff");
            let revert = matches.is_present("revert");
            let context = matches.is_present("context");
            generate::process_script(script, explain, diff, revert, context).await?;
        } else if matches.subcommand_matches("setup").is_some() {
            setup::setup_model().await?;
        } else if matches.subcommand_matches("clear").is_some() {
            commands::clear_history()?;
        } else {
            eprintln!("Invalid usage: blvflag (script.py) (--flag)");
        }
    Ok(())
} // end main

