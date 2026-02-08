use std::fs;
use std::path::PathBuf;
use chrono::Local;
use dirs_next;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RunRecord {
    pub run_id: String,
    pub timestamp: String,
    pub run_contents: String,
    pub is_error: bool,
    pub is_fixed: bool,
}

pub fn record_run(error_type: &str, script_name: &str, run_contents: &str, 
    is_error: bool, is_fixed: bool,) -> std::io::Result<()> {

    let mut base = dirs_next::home_dir().expect("Failed to get home dir");
    base.push("blvflag/tool/buckets");
    base.push(error_type);
    base.push(script_name.trim_end_matches(".py"));

    fs::create_dir_all(&base)?;

    // collect existing cycles
    let mut cycles: Vec<PathBuf> = fs::read_dir(&base)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|e| e == "json").unwrap_or(false))
        .collect();

    cycles.sort();

    // determine which cycle we want
    let cycle_path = if let Some(last_cycle) = cycles.last() {
        let data = fs::read_to_string(last_cycle)?;
        let runs: Vec<RunRecord> = serde_json::from_str(&data).unwrap_or_default();

        if runs.last().map(|r| r.is_fixed).unwrap_or(false) {
            // last cycle is FIXED?
            let idx = cycles.len() + 1;
            base.join(format!("cycle_{}.json", idx))
        } else {
            last_cycle.clone()
        }
    } else { // or make a new one
        base.join("cycle_1.json")
    };

    // load existing runs or start again
    let mut runs: Vec<RunRecord> = if cycle_path.exists() {
        serde_json::from_str(&fs::read_to_string(&cycle_path)?)?
    } else {
        vec![]
    };

    // append new run to the cycle
    runs.push(RunRecord {
        run_id: format!("run_{}", Local::now().timestamp()),
        timestamp: Local::now().to_rfc3339(),
        run_contents: run_contents.to_string(),
        is_error,
        is_fixed,
    });

    fs::write(cycle_path, serde_json::to_string_pretty(&runs)?)?;
    Ok(())
}

pub fn find_last_error_type(script_name: &str) -> Option<String> {
    let mut root = dirs_next::home_dir()?;
    root.push("blvflag/tool/buckets");

    // get all error types within the bucket dir
    let entries = fs::read_dir(&root).ok()?;

    
    for entry in entries.flatten() {
        let error_type = entry.file_name().to_string_lossy().to_string();
        let script_dir = entry
            .path()
            .join(script_name.trim_end_matches(".py"));

        if !script_dir.exists() {
            continue;
        }

        let mut cycles: Vec<PathBuf> = fs::read_dir(&script_dir)
            .ok()?
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().map(|e| e == "json").unwrap_or(false))
            .collect();

        cycles.sort();

        if let Some(last_cycle) = cycles.last() {
            let data = fs::read_to_string(last_cycle).ok()?;
            let runs: Vec<RunRecord> = serde_json::from_str(&data).ok()?;

            // iff last run is NOT fixed this is the active error
            if runs.last().map(|r| !r.is_fixed).unwrap_or(false) {
                return Some(error_type);
            }
        }
    }
    None
}

pub fn fixed_cycles(error_type: &str) -> Vec<Vec<RunRecord>> {
    let mut recovered = Vec::new();

    let mut root = dirs_next::home_dir().expect("Failed to get home dir");
    root.push("blvflag/tool/buckets");
    root.push(error_type);

    let script_dirs = match fs::read_dir(&root) {
        Ok(d) => d,
        Err(_) => return recovered,
    };

    for script_dir in script_dirs.flatten() {
        let cycle_files = match fs::read_dir(script_dir.path()) {
            Ok(d) => d,
            Err(_) => continue,
        };

        for cycle in cycle_files.flatten() {
            let data = match fs::read_to_string(cycle.path()) {
                Ok(d) => d,
                Err(_) => continue,
            };

            let runs: Vec<RunRecord> = match serde_json::from_str(&data) {
                Ok(r) => r,
                Err(_) => continue,
            };

            // find the FIX that ends the cycle
            if let Some(fix_idx) = runs.iter().rposition(|r| r.is_fixed) {
                recovered.push(runs[..=fix_idx].to_vec());
            }
        }
    }
    recovered
}

// end file.