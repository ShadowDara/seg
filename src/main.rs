use std::env;

use samfileparser::init::{ErrorMode, RunConfig};
#[cfg(windows)]
use win_utf8_rs::enable_utf8;
//use minify_html::{Cfg, minify};
use std::fs;
use std::io;

mod linksaver;
mod help;
mod birthdaytool;
// mod genicon;

use crate::help::help;

const PROGNAME: &str = "seg";

fn load_file(path: &str) -> Result<String, io::Error> {
    fs::read_to_string(path)
}


// Main function
fn main() {
    #[cfg(windows)]
    let _ = enable_utf8();

    let args: Vec<String> = env::args().collect();

    // Check arguemnt len
    if args.len() < 2 {
        // eprintln!("{}{}{}: {}No Argument Provided!{} - run with --help!", YELLOW, PROGNAME, END, RED, END);
        help();
        return;
    }

    let first_arg = &args[1];

    match first_arg.as_str() {
        // Print Help
        "-h" | "--help" => {
            help();
        }

        // Init
        "--init" => {
            samfileparser::init::init();
        }

        // Linksaver
        "--linksaver" | "-l" => {
            let mut sndarg = "";
            
            if args.len() >= 3 {
                sndarg = &args[2];
            }

            linksaver::execute(sndarg);
        }

        // // SX
        // "-x" | "--sx" => {
        //     let mut cmd = "";
        //     if args.len() >= 3 {
        //         cmd = &args[2];
        //     }

        //     load_commands();
        //     if (command_exists(cmd)) {
        //         execute_command(cmd, "");
        //     } else {
        //         println!("Command {} does not exist!", red(cmd))
        //     }
        // }

        // Birthday Tool
        "-b" | "--birthday" => {
            birthdaytool::bmain();
        }

        // // gen Icon File
        // "-g" | "--genicon" => {
        //     let mut sndarg = "";
            
        //     if args.len() >= 3 {
        //         sndarg = &args[2];
        //         let _ = genicon::genicon(sndarg);
        //         return;
        //     }

        //     println!("No arg for icon name provied!");
        // }

        // // Minify HTML
        // "-min" => {
        //     let mut sndarg = "";
            
        //     if args.len() >= 3 {
        //         sndarg = &args[2];
        //         let html = load_file(sndarg).unwrap();
        //         let mut cfg = Cfg::new();
        //         cfg.keep_comments = false;
        //         let minified = minify(&html.as_bytes(), &cfg);
        //         let _ = fs::write(sndarg, minified);
        //     }
        // }

        // When not found
        _ => {
            let conf = RunConfig{debug: true, errorMode: ErrorMode::FailFast};
            samfileparser::init::run_sam_file(first_arg, conf);
        }
    }
}
