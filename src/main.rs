use std::env;

#[cfg(windows)]
use win_utf8_rs::enable_utf8;

mod linksaver;
mod help;
mod birthdaytool;
mod samfile;

use crate::help::help;

const PROGNAME: &str = "seg";


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
            samfile::init();
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

        // When not found
        _ => {
            samfile::run_sam_file(first_arg);
        }
    }
}
