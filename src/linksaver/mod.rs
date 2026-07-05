mod add;
mod format;

use chrono::Utc;
use fluaterm::{END, ITALIC, PURPLE};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::{PathBuf},
    process::Command,
};

use crate::linksaver::{add::{add4, add5}, format::{list, view, viewx}};

const NOTE: &str = "This file was generated with linksaver from seg from the samengine project. https://samengine.js.org or https://github.com/shadowdara/seg";
const SCHEMA_URL: &str = "https://raw.githubusercontent.com/ShadowDara/samengine/refs/heads/master/.samengine/shema.linksaver.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Link {
    name: Option<String>,
    link: String,
    description: String,
    license: Option<String>,
    author: Option<String>,
    licenselink: Option<String>,
    showinlist: bool,
    changenotice: bool,
    date: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Link4 {
    link: String,
    date: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AppConfig {
    #[serde(rename = "$schema", default)]
    schema: Option<String>,

    projectname: String,
    pretty: bool,
    links: Vec<Link>,
    links2: Vec<String>,

    #[serde(default)]
    links3: Vec<String>, // Pfade zu Lizenzdateien

    // Link 2 2.0
    #[serde(default)]
    links4: Vec<Link4>,

    // Link 3 2.0
    #[serde(default)]
    links5: Vec<Link4>,

    note: Option<String>,
}

// ---------- PATH ----------

fn config_path() -> PathBuf {
    let mut path = env::current_dir().unwrap();
    path.push(".samengine/linksaver.json");
    path
}

// ---------- IO ----------

fn prompt(msg: &str) -> String {
    print!("{}", msg);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

// ---------- CONFIG ----------

fn new_config(name: String) -> AppConfig {
    AppConfig {
        schema: Some(SCHEMA_URL.to_string()),
        projectname: name,
        pretty: true,
        links: vec![],
        links2: vec![],
        links3: vec![],
        links4: vec![],
        links5: vec![],
        note: Some(NOTE.to_string()),
    }
}

fn save(config: &AppConfig) -> Result<(), std::io::Error> {
    let path = config_path();

    let json = if config.pretty {
        serde_json::to_string_pretty(config).unwrap()
    } else {
        serde_json::to_string(config).unwrap()
    };

    fs::write(path, json)?;
    Ok(())
}

fn load() -> Result<AppConfig, std::io::Error> {
    let path = config_path();

    if !path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "config not found",
        ));
    }

    let data = fs::read_to_string(path)?;
    let mut config: AppConfig = serde_json::from_str(&data)?;

    if config.projectname.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "projectname must be set",
        ));
    }

    // ✅ AUTO-INJECT $schema if missing
    if config.schema.is_none() {
        config.schema = Some(SCHEMA_URL.to_string());
    }

    config.note = Some(NOTE.to_string());

    Ok(config)
}

// ---------- COMMANDS ----------

fn init() {
    println!("Init Linksaver");

    let dir = env::current_dir().unwrap().join(".samengine");
    fs::create_dir_all(&dir).unwrap();

    let path = config_path();
    if path.exists() {
        println!("Config already exists: {:?}", path);
        return;
    }

    // Create info files
    let _datei = File::create(".samengine/links.info.md").unwrap();
    let _datei = File::create(".samengine/links.info.txt").unwrap();

    let name = prompt("Projectname: ");
    let config = new_config(name);

    save(&config).unwrap();
    println!("Created config at {:?}", path);
}

// add a multi license
fn add(config: &mut AppConfig) {
    let link = Link {
        name: {
            let v = prompt("Name (optional): ");
            if v.is_empty() { None } else { Some(v) }
        },
        link: prompt("New Link: "),
        description: prompt("New Description: "),
        author: {
            let v = prompt("Author (optional): ");
            if v.is_empty() { None } else { Some(v) }
        },
        license: {
            let v = prompt("License (optional): ");
            if v.is_empty() { None } else { Some(v) }
        },
        licenselink: {
            let v = prompt("License Link (optional): ");
            if v.is_empty() { None } else { Some(v) }
        },
        showinlist: prompt("Show in list? (y/n, default y): ") != "n",
        changenotice: prompt("Mark as changed? (y/n, default n): ") == "y",
        date: Some(Utc::now().to_rfc3339()),
    };

    config.links.push(link);
    save(config).unwrap();
    println!("Added new link!");
}

fn open_link(url: &str) {
    let cmd = if cfg!(windows) {
        Command::new("cmd").args(["/C", "start", url]).spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(url).spawn()
    } else {
        Command::new("xdg-open").arg(url).spawn()
    };

    if let Err(e) = cmd {
        eprintln!("Error opening link: {}", e);
    }
}

fn open_all(config: &AppConfig) {
    println!("Opening links...");
    for l in &config.links {
        open_link(&l.link);
    }
}

// ---------- HELP ----------

fn help() {
    println!(
        r#"
██╗     ██╗███╗   ██╗██╗  ██╗███████╗ █████╗ ██╗   ██╗███████╗██████╗ 
██║     ██║████╗  ██║██║ ██╔╝██╔════╝██╔══██╗██║   ██║██╔════╝██╔══██╗
██║     ██║██╔██╗ ██║█████╔╝ ███████╗███████║██║   ██║█████╗  ██████╔╝
██║     ██║██║╚██╗██║██╔═██╗ ╚════██║██╔══██║╚██╗ ██╔╝██╔══╝  ██╔══██╗
███████╗██║██║ ╚████║██║  ██╗███████║██║  ██║ ╚████╔╝ ███████╗██║  ██║
╚══════╝╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝  ╚═══╝  ╚══════╝╚═╝  ╚═╝
{}by samengine{}

=== {}Commands{} ===
    help    show this message
    init    create config
    add     add link
    add2    add entry (text only)
    add3    add license file
    view    formats the links into a Markdown File
    viewx   formats the links into a TXT File
    list    list links
    (none)  open all links
"#,
        ITALIC, END, PURPLE, END
    );
}

// ---------- MAIN ----------

pub fn execute(arg: &str) {
    match arg {
        "help" | "-h" | "--help" => {
            help();
            return;
        }
        "init" => {
            init();
            return;
        }
        _ => {}
    }

    let mut config = match load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Linksaver: Config Error: {}", e);
            eprintln!("Run 'init' first.");
            std::process::exit(1);
        }
    };

    match arg {
        "add" => add(&mut config),
        "add2" => add4(&mut config),
        "add3" => add5(&mut config),
        "view" => view(&config),
        "viewx" => viewx(&config),
        "list" => list(&config),
        _ => open_all(&config),
    }
}
