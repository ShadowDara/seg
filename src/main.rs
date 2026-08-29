// SEG

use samfileparser::init::{tasks, view_samfile_tasks};
use samfileparser::init::{ErrorMode, RunConfig};

use win_utf8_rs::enable_utf8;
use cargo_embed_manifest::embed;

embed!();

use samtool::buildin::BUILTIN_SAMFILE2;
use samtool::license::lmain;
use samtool::printbanner;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "seg")]
#[command(about = "seg cli tool for samfiles")]
struct Args {
    /// Zeigt die verfügbaren SAM-File Tasks
    #[arg(short = 'a', long = "all", conflicts_with_all = ["linksaver", "license"])]
    all: bool,

    /// Deprecated LinkSaver
    #[arg(short = 'l', long = "linksaver", conflicts_with_all = ["all", "license"])]
    linksaver: bool,

    /// Lizenzinformationen anzeigen
    #[arg(short = 'b', long = "license", conflicts_with_all = ["all", "linksaver"])]
    license: bool,

    /// Verbose-Ausgabe
    #[arg(short, long, default_value_t = false, global = true)]
    verbose: bool,

    /// Config path
    #[arg(
        short,
        long,
        global = true
    )]
    config: Option<String>,

    /// SAM-File / SAM-Task
    samfile: Option<String>,
}

fn main() {
    human_panic::setup_panic!();

    let _ = enable_utf8();

    let args = Args::parse();

    printbanner();

    if args.all {
        view_samfile_tasks(BUILTIN_SAMFILE2);
        return;
    }

    if args.linksaver {
        println!(
            "This option is deprecated! See here for more Infos: \
             https://shadowdara.github.io/docs/#/linksaver"
        );
        return;
    }

    if args.license {
        lmain(args.verbose, &args.config.unwrap_or("lb.config.json".to_string()));
        return;
    }

    match args.samfile {
        None => {
            //view_samfile_tasks(BUILTIN_SAMFILE2);
            tasks();
        }

        Some(samfile) => {
            let conf = RunConfig {
                debug: true,
                errorMode: ErrorMode::FailFast,
            };

            samfileparser::init::run_sam_file(
                &samfile,
                conf,
                BUILTIN_SAMFILE2,
            );
        }
    }
}
