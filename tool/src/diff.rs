use similar::{TextDiff, ChangeTag};

pub fn compare_strs(old: &str, new: &str) -> std::io::Result<String> {
    let diff = TextDiff::from_lines(old, new);
    let mut output = String::new();

      // we iterate through each change,
      // - find delete append a - like git
      // - find an insert append a + like git

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Delete => output.push_str(&format!("- {}", change)),
            ChangeTag::Insert => output.push_str(&format!("+ {}", change)),
            ChangeTag::Equal => {}
        }
    }

    Ok(output) // return
}

pub fn count_changes(old: &str, new: &str) -> usize {
    let diff = TextDiff::from_lines(old, new);

    diff.iter_all_changes()
        .filter(|c| matches!(c.tag(), ChangeTag::Delete | ChangeTag::Insert))
        .count()
}
