use regex::Regex;
use crate::diff;
use crate::buckets::RunRecord;

#[derive(Debug, Clone)]
pub struct RankedCycle {
    pub cycle: Vec<RunRecord>,
    pub score: f64,
}

pub fn extract_error_line_number(stderr: &str) -> Option<usize> {
    for line in stderr.lines() {
        if line.contains("File") && line.contains("line") {
            let parts: Vec<&str> = line.split(',').collect();

            for part in parts {
                if part.trim().starts_with("line") {
                    let number_str = part.trim().replace("line", "").trim().to_string();
                    if let Ok(num) = number_str.parse::<usize>() {
                        return Some(num);
                    }
                }
            }
        }
    }
    None
}

pub fn get_line_from_script(script: &str, line_number: usize) -> Option<String> {
     script
        .lines()
        .nth(line_number.saturating_sub(1))
        .map(|s| s.to_string())
}

fn error_line_score(current_line: &str, historical_script: &str) -> f64 {
    if current_line.trim().is_empty() { return 0.0; }

    let re = Regex::new(r"\W+").unwrap(); // regex thanks

    let current_tokens: Vec<&str> = re.split(current_line).filter(|s| !s.is_empty()).collect();
    if current_tokens.is_empty() { return 0.0; }

    let historical_tokens: Vec<&str> = re.split(historical_script).filter(|s| !s.is_empty()).collect();

    let overlap = current_tokens.iter().filter(|t| historical_tokens.contains(t)).count();

    overlap  as f64 / current_tokens.len() as f64
}

fn compute_patch_score(pre_fix: &str, post_fix: &str, current_script: &str) -> f64 {
    let hist_changes = diff::count_changes(pre_fix, post_fix);
    let curr_changes = diff::count_changes(pre_fix, current_script);

    if hist_changes == 0 { return 0.0;}

    1.0 / (1.0 + (hist_changes as f64 - curr_changes as f64).abs())
}

fn extract_vector(script: &str) -> Vec<f64> {
    vec![
        script.lines().count() as f64,
        script.matches("def ").count() as f64,
        script.matches("if ").count() as f64,
        script.len() as f64,
    ]
}

fn dot_product(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() { return 0.0; }
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() { return 0.0; }

    let dot = dot_product(a, b);
    let norm_a = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b = b.iter().map(|x| x * x).sum::<f64>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 { return 0.0; }

    dot / (norm_a * norm_b)
}

pub fn generate_ranking(current_error_line: &str, current_script: &str, 
    cycles: Vec<Vec<RunRecord>>,) -> Vec<Vec<RunRecord>> {

    if cycles.is_empty() { return vec![]; }

    let current_vector = extract_vector(current_script);
    let mut ranked_cycles: Vec<RankedCycle> = Vec::new();

    for cycle in cycles.into_iter() {
        if cycle.len() < 2 { continue; }

        let pre_fix = &cycle[cycle.len() - 2].run_contents; // pre before
        let post_fix = &cycle[cycle.len() - 1].run_contents; // post current

        let line_score = error_line_score(current_error_line, pre_fix);

        let patch_score = compute_patch_score(pre_fix, post_fix, current_script);

        let historical_vector = extract_vector(pre_fix);
        let feature_score = cosine_similarity(&current_vector, &historical_vector);

        // favor structual similarity
        let score = 0.35 * line_score + 0.30 * patch_score + 0.35 * feature_score;
       
        /* debug city
        println!("{:?}", cycle);
        println!("Line Score {}", line_score);
        println!("Patch Score {}", patch_score);
        println!("Feature Score {}", feature_score);
        println!("Overall Score {}", score);
        */

        ranked_cycles.push(RankedCycle {
            cycle,
            score,
        });
    }

    ranked_cycles.sort_by(|a, b|
        b.score.partial_cmp(&a.score).unwrap()
    );

    ranked_cycles.into_iter().take(2).map(|r| r.cycle).collect()
}