use crate::commands;
use crate::diff;
use crate::buckets;
use crate::model;
use crate::ranking;

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use chrono::Local;
use dirs_next;
use crate::setup::ensure_dirs;


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

    ensure_dirs()?;

    match out {
            //Standard Out
            Ok((commands::OutputType::Stdout, output)) => { 

                if !diff { println!("{}", output); } // if we dont have a flag just give back

                let script_name = Path::new(script_path).file_name().unwrap().to_string_lossy().to_string();
                let date_stamp = Local::now().format("%Y%m%d_%H%M%S").to_string();

                let mut history_dir: PathBuf = dirs_next::home_dir().expect("Failed to get home directory");
                history_dir.push("blvflag/tool/history/std_history"); // create the std_history dir if it DOESNT EXIST
                fs::create_dir_all(&history_dir)?;
            
                let json_name = format!("{}_{}.json", script_name.trim_end_matches(".py"), date_stamp); // dating format
                let full_path = history_dir.join(&json_name); 
                let current_script_content = fs::read_to_string(script_path)?; // stringify all contents 

                 // Hashing Logic
                if let Some(error_type) = buckets::find_last_error_type(&script_name) { // matches to write to proper cycle
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
    
                                let second_date_stamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
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

                if !diff && !explain && !context {
                    println!("Error Caught! Use --explain OR --diff for help.\n"); // TODO might change
                    println!("{}", error_output);
                }
            
                let script_name = Path::new(script_path).file_name().unwrap().to_string_lossy().to_string();
                 let date_stamp = Local::now().format("%Y%m%d_%H%M%S").to_string();

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

                let auto_context = buckets::identical_error(&error_type, &script_name);

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
    
                                let second_date_stamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
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
                            // call model
                            let explanation = model::call_llm(prompt).await?;
                            println!("Error Explanation:\n{}", explanation);

                    } // end explain

                   //context checks
                   let fixed_cycles = buckets::fixed_cycles(&error_type);
                   let manual_context = context;
                   let automatic_context = auto_context && !fixed_cycles.is_empty();

                    if manual_context || automatic_context {
                        // NO FIXED - explain fall back
                        if fixed_cycles.is_empty() {
                            let prompt = format!(
                                "Provide the error line number and explain in a compact screen readable format for \
                                blind-low-vision programmers. Here is the error:\n{}",
                                error_output
                            );

                            let explanation = model::call_llm(prompt).await?;
                            println!("No Context Found!\n\nFalling Back to --explain:\n{}", explanation);
                            return Ok(());
                        }

                        // 1 get the specific error producing line
                        let error_line_number = ranking::extract_error_line_number(&error_output);

                        let failing_line = if let Some(line_num) = error_line_number {
                            ranking::get_line_from_script(&current_script_content, line_num)
                                .unwrap_or_else(|| "".to_string())
                        } else {
                            "".to_string()
                        };

                        // 2 get the ranking of associated cycles
                        let ranked_cycles = ranking::generate_ranking(
                            &failing_line,
                            &current_script_content,
                            fixed_cycles,
                        );
                        
                        // NO RANKED - explain fall back
                        if ranked_cycles.is_empty() {
                            let prompt = format!(
                                "Provide the error line number and explain in a compact screen readable format for \
                                blind-low-vision programmers. Here is the error:\n{}",
                                error_output
                            );

                            let explanation = model::call_llm(prompt).await?;
                            println!("No Relevant Context Found!\n\nFalling Back to --explain:\n{}", explanation);
                            return Ok(());
                        }

                        // 3 take top two rankigns only
                        let top_cycles = ranked_cycles.into_iter().take(2);
                        let mut historical_fixed_run_contents = String::new();

                        // 4 - process and prepare for the llm prompt
                        for cycle in top_cycles {
                            if cycle.len() < 2 { continue; }

                            let pre_fix = &cycle[cycle.len() - 2];
                            let fixed = &cycle[cycle.len() - 1];

                            historical_fixed_run_contents.push_str(
                                &format!("\n[before fix]\n{}\n", pre_fix.run_contents)
                            );

                            historical_fixed_run_contents.push_str(
                                &format!("\n[after fix]\n{}\n", fixed.run_contents)
                            );
                        }

                       let prompt = format!(
                            "You are assisting a blind or low-vision programmer.\n\
                            IMPORTANT OUTPUT RULES (MUST FOLLOW):\n\
                            - Use CLEAR SECTION HEADERS\n\
                            - Use SHORT BULLET POINTS\n\
                            - NO PARAGRAPHS\n\
                            - Speak directly to the user\n\
                            - Do NOT provide the full solution\n\
                            ====================\n\
                            CURRENT ERROR\n\
                            ====================\n\
                            Error type:\n\
                            {error_type}\n\
                            Error message:\n\
                            {error_message}\n\
                            ====================\n\
                            CURRENT SCRIPT\n\
                            ====================\n\
                            {current_script}\n\
                            ====================\n\
                            RELEVANT PREVIOUS FIXES\n\
                            ====================\n\
                            {historical_cycles}\n\
                            ====================\n\
                            RESPONSE FORMAT\n\
                            ====================\n\
                            WHAT IS GOING WRONG\n\
                            - Explain root cause\n\
                            WHAT YOU DID BEFORE\n\
                            - Summarize previous fix\n\
                            WHAT TO TRY NEXT\n\
                            - 2–4 short hints\n\
                            BEGIN RESPONSE NOW.",
                                    error_type = error_type,
                                    error_message = error_output,
                                    current_script = current_script_content,
                                    historical_cycles = historical_fixed_run_contents
                        );

                        let context_response = model::call_llm(prompt).await?;

                        if auto_context && !manual_context {
                            println!("Automatic Context:\n\n{}", context_response);
                        } else {
                            println!("Context:\n\n{}", context_response);
                        }
                    }
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
// end file.
