// TODO: Need bucket searching logic in here somewhere
// FINAL STEP: combine with bucket.rs to make context.rs file

use serde::{Serialize, Deserialize}; // json writes
use regex::Regex; 
use std::fs; // files
use chrono::Local; // time
use dirs_next; // files

#[derive(Serialize, Deserialize)]
pub struct Node {
    pub state: String,     // s0, ERROR, FIXED >> may eventually change ERROR to be the exact type of ERROR.
    pub file: String,
    pub line: usize,
    pub function: String,
    pub message: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorGraph {
    pub script_name: String,
    pub error_type: String,
    pub error_list: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<(usize, usize)>,
}


impl ErrorGraph {
    pub fn load_or_new(script_name: &str) -> Self {
        // creates or get the graphs dir 
        let mut graph_path = dirs_next::home_dir().unwrap();
        graph_path.push("blvflag/tool/history/graphs");
        fs::create_dir_all(&graph_path).unwrap();

        // set up the file
        let graph_file = graph_path.join(format!("{}_graph.json", script_name.trim_end_matches(".py")));

        // load
        if graph_file.exists() {
            if let Ok(contents) = fs::read_to_string(&graph_file) {
                if let Ok(graph) = serde_json::from_str::<ErrorGraph>(&contents) {
                    return graph; // return out dont make new
                }
            }
        }
        
        // new
        ErrorGraph {
            script_name: script_name.to_string(),
            error_type: "ROOT".into(),
            error_list: "".into(),
            nodes: vec![Node {
                state: "s0".into(),
                file: script_name.into(),
                line: 0,
                function: "main".into(), // assume main
                message: "Initial state".into(),
                timestamp: Local::now().to_rfc3339(),
            }],
            edges: vec![], // s0 has no outgoing edges yet so empty
        }
    } // end load_new

    pub fn add_state(&mut self, stderr_or_out: &str, fixed: bool) {
        // get the error type
        let (etype, msg) = Self::extract_error_type(stderr_or_out);
        let re = Regex::new(r#"File "(.+)", line (\d+), in (.+)"#).unwrap();

        let mut file = self.script_name.clone(); // default to script name for now, will look later
        let mut line = 0; // default to 0 will look later
        let mut function = "main".into(); // defualt to main, we look at it later

        // iterate through regex stuff
        for cap in re.captures_iter(stderr_or_out) {
            file = cap[1].to_string();
            line = cap[2].parse::<usize>().unwrap_or(0);
            function = cap[3].to_string();
        }

        let state = if fixed {"FIXED"} else {"ERROR"};

        // put it all together
        let new_node = Node {
            state: state.into(),
            file,
            line,
            function,
            message: msg,
            timestamp: Local::now().to_rfc3339(),
        };

        // and add it to our Nodes vector
        let preva = self.nodes.len() - 1;
        self.nodes.push(new_node);
        let newa = self.nodes.len() - 1;
        
        // TODO make more intuitive
        // make the simple edges 
        self.edges.push((preva, newa)); 

        if !fixed {
            if !self.error_list.contains(&etype) {
                if !self.error_list.is_empty() {
                    self.error_list.push_str(" | ");
                }
                self.error_list.push_str(&etype);
            }
            self.error_type = etype;
        }
    } // end add state

    // is this name obvious enough?
    fn extract_error_type(stderr: &str) -> (String, String) {
        if let Some(last_line) = stderr.lines().last() {
            if let Some((etype, msg)) = last_line.split_once(": ") {
                return (etype.trim().to_string(), msg.trim().to_string()); // ok
            }
        }
        // Will need a bucket for this consider
        ("UnknownError".into(), stderr.trim().to_string())
    }

    pub fn save(&self) -> std::io::Result<()> {
        // create or get the graphs dir
        let mut graph_dir = dirs_next::home_dir().unwrap();
        graph_dir.push("blvflag/tool/history/graphs");
        fs::create_dir_all(&graph_dir)?;

        let file_name = format!(
            "{}_graph.json",
            self.script_name.trim_end_matches(".py")
        );
        let full_path = graph_dir.join(file_name);

        // this is beautiful..
        let serialized = serde_json::to_string_pretty(&self)?;
        fs::write(&full_path, serialized)?;
        Ok(())
    }

} // end class
