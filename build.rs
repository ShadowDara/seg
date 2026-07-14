use std::{fs, path::Path};

fn main() {
    let input_path = "buildin.samfile";

    let input = fs::read_to_string(input_path)
        .expect("failed to read buildin.samfile");

    // remove comments starting with #
    let filtered: String = input
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            !trimmed.starts_with('#') && !trimmed.trim().is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with("--")
        })
        .collect::<Vec<_>>()
        .join("\n");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("builtin_filtered.rs");

    fs::write(
        &out_path,
        format!(
            "pub const BUILTIN_SAMFILE2: &str = r#\"{}\"#;",
            filtered
        ),
    )
    .expect("failed to write filtered samfile");

    println!("cargo:rerun-if-changed={}", input_path);
}
