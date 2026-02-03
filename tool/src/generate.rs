use crate::commands;
use crate::diff;
use crate::buckets;

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use chrono::Local;
use dirs_next;

/* metric testing
use std::time::Instant;
use std::fs::OpenOptions;
use std::io::Write;
use serde_json::json;
*/

pub async fn process_script(script_path: &str, explain: bool, diff: bool, revert: bool, context: bool,) -> Result<(), Box<dyn Error>> {
    let out = commands::run_script(script_path);

    /* metric testing
    let start = Instant::now();
    */

    match out {
            //Standard Out
            Ok((commands::OutputType::Stdout, output)) => { 

                if !diff { println!("{}", output); } // if we dont have a flag just give back

                let script_name = Path::new(script_path).file_name().unwrap().to_string_lossy().to_string();
                let date_stamp = Local::now().to_string();

                let mut history_dir: PathBuf = dirs_next::home_dir().expect("Failed to get home directory");
                history_dir.push("blvflag/tool/history/std_history"); // create the std_history dir if it DOESNT EXIST
                fs::create_dir_all(&history_dir)?;
            
                let json_name = format!("{}_{}.json", script_name.trim_end_matches(".py"), date_stamp); // dating format
                let full_path = history_dir.join(&json_name); 
                let current_script_content = fs::read_to_string(script_path)?; // stringify all contents 

                 // Hashing Logic
                if let Some(error_type) = find_last_error_type(&script_name) { // matches to write to proper cycle
                    let _ = buckets::record_run(
                        &error_type,
                        &script_name,
                        &current_script_content,
                        false, // is_error
                        true,  // is_fixed
                    );
                 }
                 // ---


                /*
                    Here we are going to read through both of the history sub dirs, std_history & err_history.
                    This allows use to get the MOST recent file that has been saved from our check. It's going
                    to save regardless of std_out or std_err.
                */

                let prefix = script_name.trim_end_matches(".py");
                let mut all_versions: Vec<PathBuf> = vec![];
            
                    let std_history_dir = dirs_next::home_dir().unwrap().join("blvflag/tool/history/std_history");
                    let std_versions = fs::read_dir(&std_history_dir)? // read the users std dir
                        .filter_map(|entry| {
                            let path = entry.ok()?.path();
                            let fname = path.file_name()?.to_string_lossy();
                            if fname.starts_with(prefix) {
                                Some(path) 
                            } else { 
                                None // TODO unsure if this is good practice.
                            }
                        }); // end std read
            
                    let err_history_dir = dirs_next::home_dir().unwrap().join("blvflag/tool/history/err_history");
                    let err_versions = fs::read_dir(&err_history_dir)? // read the users std out dir
                        .filter_map(|entry| {
                            let path = entry.ok()?.path();
                            let fname = path.file_name()?.to_string_lossy();
                            if fname.starts_with(prefix) { 
                                Some(path) 
                            } else { 
                                None 
                            }
                        }); // end err read
            
                    all_versions.extend(std_versions); // append std
                    all_versions.extend(err_versions); // append err
                    all_versions.sort(); // sort to get the most recent

                    // add the revert flag 
                    if revert {
                        if all_versions.len() < 2 {
                            println!("No previous version to revert to!");
                        } else {
                            let latest = &all_versions[all_versions.len() - 1];      // this is most recent
                            let previous = &all_versions[all_versions.len() - 2];    // this is one b4 latest

                            let prev_content = fs::read_to_string(previous)?;
                            fs::write(script_path, &prev_content)?; // write back revert

                            fs::remove_file(latest)?; // delete most recent

                            println!( "Reverted script {} to {:?} and removed {:?}", script_path, previous, latest);
                        }
                        return Ok(()); // stop
                    }

                    let mut should_save = true;
                    if let Some(last_version) = all_versions.last() {
                        let previous_content = fs::read_to_string(last_version)?; // stringify for compare
                        if previous_content != current_script_content {
                            should_save = true;
                        } else {
                            should_save = false;
                        }
                    } 
                    
                    if should_save { // write to the path the contents, love this C like syntax
                        fs::write(&full_path, &current_script_content)?;
                        println!("\nAuto-Saving contents to: {:?}", full_path);
                    }
                
                    //FLAGS:
                    if diff {
                        if !all_versions.is_empty() {
                            let last_version = all_versions.last().unwrap();
                            let last_content = fs::read_to_string(last_version)?;
                            let mut diff_output = diff::compare_strs(&last_content, &current_script_content)?;
                        
                            if diff_output.is_empty() && all_versions.len() >= 2 {
                                let second_last_version = &all_versions[all_versions.len() - 2];
                                let second_last_content = fs::read_to_string(second_last_version)?;
                                diff_output = diff::compare_strs(&second_last_content, &current_script_content)?;
                            }
                        
                            if diff_output.is_empty() {
                                println!("No changes made.");
                            } else {
                                println!("------changes------");
                                println!("{}", diff_output);
                                println!("-------------------");
                                
                                fs::write(&full_path, &current_script_content)?;
    
                                let second_date_stamp = Local::now().to_string(); 
                                let second_json_name = format!("{}_{}.json", script_name.trim_end_matches(".py"), second_date_stamp);
                                let second_full_path = history_dir.join(&second_json_name);
                                fs::write(&second_full_path, &current_script_content)?;
                
                            }
                        } else {
                            println!("No prior version found to diff against.");
                        }
                    }
            } // end Standard Out -------------------------------
            
            // Standard Error
             Ok((commands::OutputType::Stderr, error_output)) => { 

                if !diff && !explain {
                    println!("Error Caught! Use --explain OR --diff for help.\n"); // TODO might change
                    println!("{}", error_output);
                }
            
                let script_name = Path::new(script_path).file_name().unwrap().to_string_lossy().to_string();
                let date_stamp = Local::now().to_string();

                let mut history_dir: PathBuf = dirs_next::home_dir().expect("Failed to get home directory");
                history_dir.push("blvflag/tool/history/err_history"); // create the err_history dir if it DOESNT EXIST
                fs::create_dir_all(&history_dir)?;
            
                let json_name = format!("{}_{}.json", script_name.trim_end_matches(".py"), date_stamp); // dating format
                let full_path = history_dir.join(&json_name); 
                let current_script_content = fs::read_to_string(script_path)?; // stringify all contents 

                // Hashing Logic ---
                fn get_error(stderr: &str) -> String {
                    stderr.lines().rev().find(|line| line.contains(':')).and_then(|line| line
                        .split(':').next()).unwrap_or("UnknownError").trim().to_string()
                }                                  // TODO - Bucket.rs
                                                    
                let error_type = get_error(&error_output); // find what we've got

                let _ = buckets::record_run(
                    &error_type,
                    &script_name,
                    &current_script_content,
                    true,   // is_error
                    false,  // is_fixed
                );
                // ---

                let prefix = script_name.trim_end_matches(".py");
                let mut all_versions: Vec<PathBuf> = vec![];
            
                    let std_history_dir = dirs_next::home_dir().unwrap().join("blvflag/tool/history/std_history");
                    let std_versions = fs::read_dir(&std_history_dir)? // read the users std dir
                        .filter_map(|entry| {
                            let path = entry.ok()?.path();
                            let fname = path.file_name()?.to_string_lossy();
                            if fname.starts_with(prefix) {
                                Some(path) 
                            } else { 
                                None // TODO unsure if this is good practice.
                            }
                        }); // end std read
            
                    let err_history_dir = dirs_next::home_dir().unwrap().join("blvflag/tool/history/err_history");
                    let err_versions = fs::read_dir(&err_history_dir)? // read the users std out dir
                        .filter_map(|entry| {
                            let path = entry.ok()?.path();
                            let fname = path.file_name()?.to_string_lossy();
                            if fname.starts_with(prefix) { 
                                Some(path) 
                            } else { 
                                None 
                            }
                        }); // end err read
            
                    all_versions.extend(std_versions); // append std
                    all_versions.extend(err_versions); // append err
                    all_versions.sort(); // sort to get the most recent

                    // add the revert flag
                    if revert {
                        if all_versions.len() < 2 {
                            println!("No previous version to revert to!");
                        } else {
                            let latest = &all_versions[all_versions.len() - 1];      // this is most recent
                            let previous = &all_versions[all_versions.len() - 2];    // this is one b4 latest

                            let prev_content = fs::read_to_string(previous)?;
                            fs::write(script_path, &prev_content)?; // write back revert

                            fs::remove_file(latest)?; // delete most recent

                            println!( "Reverted script {} to {:?} and removed {:?}", script_path, previous, latest);
                        }
                        return Ok(()); // stop
                    }

                    let mut should_save = true;
                    if let Some(last_version) = all_versions.last() {
                        let previous_content = fs::read_to_string(last_version)?; // stringify for compare
                        if previous_content != current_script_content {
                            should_save = true;
                        } else {
                            should_save = false;
                        }
                    } 
                    
                    if should_save { // write to the path the contents, love this C like syntax
                        fs::write(&full_path, &current_script_content)?;
                        println!("\nAuto-Saving contents to: {:?}", full_path);
                    }
                
                    //FLAGS:
                    if diff { 
                        if !all_versions.is_empty() {
                            let last_version = all_versions.last().unwrap();
                            let last_content = fs::read_to_string(last_version)?;
                            let mut diff_output = diff::compare_strs(&last_content, &current_script_content)?;
                        
                            if diff_output.is_empty() && all_versions.len() >= 2 {
                                let second_last_version = &all_versions[all_versions.len() - 2];
                                let second_last_content = fs::read_to_string(second_last_version)?;
                                diff_output = diff::compare_strs(&second_last_content, &current_script_content)?;
                            }
                        
                            if diff_output.is_empty() {
                                println!("No changes made.");
                            } else {
                                println!("------changes------");
                                println!("{}", diff_output);
                                println!("-------------------");
                                
                                fs::write(&full_path, &current_script_content)?;
    
                                let second_date_stamp = Local::now().to_string(); 
                                let second_json_name = format!("{}_{}.json", script_name.trim_end_matches(".py"), second_date_stamp);
                                let second_full_path = history_dir.join(&second_json_name);
                                fs::write(&second_full_path, &current_script_content)?;
                
                            }
                        } else {
                            println!("No prior version found to diff against.");
                        }
                    }
                    
                    if explain { 

                        let mut identical_diff_count = 0;
                        let mut prev_diff: Option<String> = None;
                    
                        let num_versions = all_versions.len();
                        let max_checks = 3.min(num_versions.saturating_sub(1)); // max checks are three back
                    
                       if num_versions >= 2 {
                            for i in (num_versions - max_checks)..(num_versions - 1) {
                                let older = fs::read_to_string(&all_versions[i])?;
                                let newer = fs::read_to_string(&all_versions[i + 1])?;
                                let diff_result = diff::compare_strs(&older, &newer)?;

                                if let Some(prev) = &prev_diff {
                                    if diff_result == *prev {
                                        identical_diff_count += 1;
                                    } else {
                                        break;
                                    }
                                } else {
                                    prev_diff = Some(diff_result);
                                }
                            }
                        }

                        let prompt = if identical_diff_count >= 2 { // if we have more than 2 repeats, we give this prompt
                            format!( 
                                "This script has failed multiple times even with the same changes. Please Provide the error line number, \
                                and offer an alternative solution or suggest a different debugging approach in a compact screen readable format for \
                                blind-low-vision programmers. Here is the error:\n{}",
                                error_output 
                            )
                        } else { // else give the original prompt
                            format!(
                                "Provide the error line number and explain in a compact screen readable format for \
                                blind-low-vision programmers. Here is the error:\n{}",
                                error_output
                            )
                        };
                    
                        let api_key_path: PathBuf = dirs_next::home_dir()
                            .expect("Could not find home dir") // first we get the api key from the folder we setup
                            .join("blvflag/tool/key/api_key");

                        if !api_key_path.exists() {
                            println!("\nCannot run --explain! Missing API key file at {:?}", api_key_path);
                            println!("Run `blvflag setup` to initialize API key file. Or help.\n");
                            return Ok(()); 
                        }
                        
                        let llama_api_key = fs::read_to_string(&api_key_path) 
                            .expect("Failed to read API key")
                            .trim()
                            .to_string(); 

                            let client = reqwest::Client::new();
                            let response = client // set up like in docuemntations, did have to modify for HTTP requests
                                .post("https://api.llama.com/v1/chat/completions") 
                                .header("Authorization", format!("Bearer {}", llama_api_key))
                                .header("Content-Type", "application/json") // its a JSON format
                                .json(&serde_json::json!({
                                    "model": "Llama-4-Maverick-17B-128E-Instruct-FP8", // same as PC's stuff
                                    "messages": [
                                        { "role": "system", "content": prompt }, // system uses the prompt we generate above
                                         { "role": "user", "content": error_output } // user gives the error
                                    ],
                                }))
                                .send()
                                .await?;

                            let json_response: serde_json::Value = response.json().await?;

                            let explanation = json_response["completion_message"]["content"]["text"] // now we parse the json in this format
                                .as_str()
                                .unwrap_or("Error communicating with Llama! \n Check ~/blvflag/tool/key/api_key OR run setup.") // incase we recieve nothing we alert
                                .to_string();

                            println!("Error Explanation:\n{}", explanation);
                    } // end explain


                    if context {
                        /*
                            1. Hash to get context
                            2. Pass to LLM like above
                        */

                        /*
                            let prompt = 
                                format!(
                                "A blind low-vision developer is struggling to fix an error. If they have fixed this error before 
                                development steps will be provided. If they have not fixed this error before this will be labeled below.
                                
                                CURRENT ERROR:
                                Error Type: {{CURRENT_ERROR_TYPE}}
                                Error Message: {{CURRENT_ERROR_MESSAGE}}

                                CURRENT SCRIPT CONTENTS:
                                {{FULL SCRIPT CONTENTS}}

                                PREVIOUSLY FIXED SCRIPTS
                                Below is a list of development cycles where this user fixed, {{CURRENT_ERROR_TYPE}}
                                CYCLES:
                                {{HISTORICAL_FIXED_RUN_CONTENTS}} 
                                {{HISTORICAL_FIXED_RUN_CONTENTS}}  
                                    . . . 

                                YOUR TASK
                                * IN A SCREEN-READER FRIENDLY FORMAT, speaking to the USER (you did this , you should do this)* 
                                * IN SIMPLE & SHORT BULLET POINTS *
                                1. Explain what is causing the current error.
                                2. Describe how the user fixed this error in previous scripts (IF APPLICABLE)
                                3. Suggest hints in order to fix the CURRENT ERROR PRODUCING SCRIPT, without providing the answer."    
                                );
                        */
                    } // end context

            } // end stderr match block -------------------------------
        Err(_) => {
            eprintln!("\nFailed to execute the script. Use -help for help");
        }
    } // match

    /* Metric testing
    let duration = start.elapsed().as_secs_f64();
    let log_path = "/Users/rylan/blvflag/tool/logs/timings.jsonl";
    let log_entry = json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "script": script_path,
        "duration_sec": duration
    });

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .expect("Failed to open timing log");
        writeln!(file, "{}", log_entry).unwrap();
    */            
                            
    Ok(()) 
} // end processing script

 // Error Collection & Hashing
fn find_last_error_type(script_name: &str) -> Option<String> {
    let mut root = dirs_next::home_dir()?;
    root.push("blvflag/tool/buckets");

    // get all error types within the bucket dir
    let entries = fs::read_dir(&root).ok()?;

    for entry in entries.flatten() {
        let error_type = entry.file_name().to_string_lossy().to_string();
        // find the associated script
        let script_dir = entry.path().join(script_name.trim_end_matches(".py")); 

        if !script_dir.exists() {
            continue;
        }

        // collect all developmetn cycles for the script
        let mut cycles: Vec<PathBuf> = fs::read_dir(&script_dir)
            .ok()?
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().map(|e| e == "json").unwrap_or(false))
            .collect();

        cycles.sort();

        // find the last, this is the one to be udpated
        if let Some(last_cycle) = cycles.last() {
            let data = fs::read_to_string(last_cycle).ok()?;
            let runs: Vec<buckets::RunRecord> =
                serde_json::from_str(&data).ok()?;

            if runs.last().map(|r| !r.is_fixed).unwrap_or(false) {
                return Some(error_type);
            }
        }
    }
    None // either no cycles or all cycles are in a FIXED state. 
         // (logic in buckets.rs will create new cycle in this case)
} // end error type

// end file.