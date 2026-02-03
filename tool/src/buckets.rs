use std::fs;
use std::path::PathBuf;
use chrono::Local;
use dirs_next;
use serde::{Serialize, Deserialize};

// TODO -- UNKNOWN ERROR BUCKET HANDLING -- GENERATE LINE 183 (180s)

// fields within our JSON
#[derive(Serialize, Deserialize)]
pub struct RunRecord {
    pub run_id: String,
    pub timestamp: String,
    pub run_contents: String,
    pub is_error: bool,
    pub is_fixed: bool,
}

// storage function
pub fn record_run(error_type: &str, script_name: &str, run_contents: 
    &str, is_error: bool,is_fixed: bool,) -> std::io::Result<()> {

    let mut root = dirs_next::home_dir().expect("Failed to get home dir");
    root.push("blvflag/tool/buckets");
    let mut base = root;
    base.push(error_type);
    base.push(script_name.trim_end_matches(".py"));

    fs::create_dir_all(&base)?; // create our storage location

    // filter through cycles
    let mut cycles: Vec<PathBuf> = fs::read_dir(&base)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|e| e == "json").unwrap_or(false))
        .collect();

    cycles.sort();

    // find and write to most recent cylcle
    let cycle_path = if let Some(last_cycle) = cycles.last() {
        let data = fs::read_to_string(last_cycle)?;
        let runs: Vec<RunRecord> =
            serde_json::from_str(&data).unwrap_or_default();

        // if the last cycle we found already has a fix - start a new one
        if runs.last().map(|r| r.is_fixed).unwrap_or(false) {
            let idx = cycles.len() + 1;
            base.join(format!("cycle_{}.json", idx))
        } else {
            last_cycle.clone()
        }
    } else {
        base.join("cycle_1.json") // default for first cycle 
    };

    // load or create a particular cycle
    let mut runs: Vec<RunRecord> = if cycle_path.exists() {
        serde_json::from_str(&fs::read_to_string(&cycle_path)?)?
    } else {
        vec![]
    };

    // append the run and it's values -- this includes fixed runs
    runs.push(RunRecord {
        run_id: format!("run_{}", Local::now().timestamp()),
        timestamp: Local::now().to_rfc3339(),
        run_contents: run_contents.to_string(),
        is_error,
        is_fixed,
    });

    fs::write(&cycle_path, serde_json::to_string_pretty(&runs)?)?;

    Ok(())
}

