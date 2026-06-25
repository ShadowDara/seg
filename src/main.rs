use std::{collections::HashMap, env, fs, path::Path};

use samfileparser::{parse, run_task, validate_all, RuntimeState};

#[cfg(windows)]
use win_utf8_rs::enable_utf8;

mod linksaver;
mod help;
mod birthdaytool;

use crate::help::help;
// use ::sx::{load_commands, command_exists, execute_command};

const PROGNAME: &str = "samtool";


// Run sth from the samfile
fn run_sam_file(command: &str) {
    let mut state = RuntimeState {
        cwd: std::env::current_dir().unwrap(),
        env: HashMap::new(),
    };

    let content = match std::fs::read_to_string(".samengine/samfile") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error while reading samfile: {}", e);
            return;
        }
    };

    let tasks = parse(&content);

    // Check for cycled dependencies
    validate_all(&tasks);

    // Map which one was already visited
    let mut visited = std::collections::HashSet::new();

    // Execute the Task
    run_task(&tasks, command, &mut visited, &mut state);
}

fn has_gitignore(dir: &str) -> bool {
    Path::new(dir).join(".gitignore").exists()
}

fn read_gitignore(dir: &str) -> Option<String> {
    let path = std::path::Path::new(dir).join(".gitignore");

    fs::read_to_string(path).ok()
}

fn is_samfile_ignored(gitignore_content: &str) -> bool {
    gitignore_content
        .lines()
        .any(|line| line.trim() == "samfile")
}

// Create new samfile
fn init() {
    let dir = std::path::Path::new(".samengine");
    let file = dir.join("samfile");

    // check first if exists
    if dir.exists() && file.exists() {
        println!("samefile already exists — aborting init");
        return;
    }

    println!("Creating a new samfile!");

    // Create .samengine Directory
    std::fs::create_dir_all(dir)
        .expect("failed to create directory");

    std::fs::write(
        &file,
        "# A new samfile, write your scripts here"
    )
    .expect("failed to create file");

    let dir2 = std::env::current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // Check if there is a gitignore
    if has_gitignore(&dir2) {
        if let Some(content) = read_gitignore(&dir2) {
            if is_samfile_ignored(&content) {
                println!("samfile is ignored by git");
            } else {
                println!("samfile is NOT ignored");
            }
        }
    }
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
            init();
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
            run_sam_file(first_arg);
        }
    }
}
