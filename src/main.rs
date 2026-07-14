use std::env;

use samfileparser::init::{tasks, view_samfile_tasks};
use samfileparser::init::{ErrorMode, RunConfig};

#[cfg(windows)]
use win_utf8_rs::enable_utf8;

mod buildin;
use crate::buildin::BUILTIN_SAMFILE2;

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
https://shadowdara.github.io/docs/#/samfile
"#);
}

// Main function
fn main() {
    #[cfg(windows)]
    let _ = enable_utf8();

    let args: Vec<String> = env::args().collect();

    printbanner();

    // Check arguemnt len
    if args.len() < 2 {
        tasks();
        return;
    }

    let first_arg = &args[1];

    if first_arg == "-a" {
        view_samfile_tasks(BUILTIN_SAMFILE2);
        return;
    }
    else if first_arg == "-l" {
        println!("This option is deprecated! See here for more Infos: https://shadowdara.github.io/docs/#/linksaver");
        return;
    }

    let conf = RunConfig {
        debug: true,
        errorMode: ErrorMode::FailFast,
    };

    samfileparser::init::run_sam_file(first_arg, conf, BUILTIN_SAMFILE2);
}
