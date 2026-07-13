use std::env;

use samfileparser::init::view_samfile_tasks;
use samfileparser::init::{ErrorMode, RunConfig};
#[cfg(windows)]
use win_utf8_rs::enable_utf8;
//use minify_html::{Cfg, minify};


fn printbanner() {
    // https://www.asciiart.eu/text-to-ascii-art
    // DOS Rebel

    println!(r#"
  █████████  ██████████   █████████ 
 ███░░░░░███░░███░░░░░█  ███░░░░░███
░███    ░░░  ░███  █ ░  ███     ░░░ 
░░█████████  ░██████   ░███         
 ░░░░░░░░███ ░███░░█   ░███    █████
 ███    ░███ ░███ ░   █░░███  ░░███ 
░░█████████  ██████████ ░░█████████ 
 ░░░░░░░░░  ░░░░░░░░░░   ░░░░░░░░░  

The runner for samfiles
https://shadowdara.github.io/docs/#/samfile"#);
}

// Main function
fn main() {
    #[cfg(windows)]
    let _ = enable_utf8();

    let args: Vec<String> = env::args().collect();

    printbanner();

    // Check arguemnt len
    if args.len() < 2 {
        view_samfile_tasks();
        return;
    }

    let first_arg = &args[1];

    let conf = RunConfig {
        debug: true,
        errorMode: ErrorMode::FailFast,
    };
    samfileparser::init::run_sam_file(first_arg, conf);
}
