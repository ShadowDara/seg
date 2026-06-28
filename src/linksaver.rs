use fluaterm::{END, ITALIC, PURPLE};
use serde::{Deserialize, Serialize};
use std::{
    env, fs::{self, File}, io::{self, Write}, path::{Path, PathBuf}, process::Command,
};

const NOTE: &str = "This file was generated with linksaver from seg from the samengine project. https://samengine.js.org or https://github.com/shadowdara/seg";

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
}

#[derive(Serialize, Deserialize, Debug)]
struct AppConfig {
    projectname: String,
    pretty: bool,
    links: Vec<Link>,
    links2: Vec<String>,

    #[serde(default)]
    links3: Vec<String>, // Pfade zu Lizenzdateien
    note: Option<String>
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
        projectname: name,
        pretty: true,
        links: vec![],
        links2: vec![],
        links3: vec![],
        note: Some(NOTE.to_string())
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
    };

    config.links.push(link);
    save(config).unwrap();
    println!("Added new link!");
}


// add a full string
fn add2(config: &mut AppConfig) {
    let entry = prompt("Entry text: ");
    config.links2.push(entry);
    save(config).unwrap();
    println!("Added new entry!");
}


// add the path to a license file
fn add3(config: &mut AppConfig) {
    let path = prompt("License file: ");

    if !Path::new(&path).exists() {
        println!("Warning: '{}' does not exist.", path);
    }

    config.links3.push(path);

    save(config).unwrap();
    println!("Added license file!");
}

/// Function to convert the links file into a Markdown file
fn view(config: &AppConfig) {
    let mut file = File::create(".samengine/links.md").unwrap();

    write!(file, "# Links for {}\n\n", config.projectname).unwrap();

    // Add Info Message
    let pfad = Path::new(".samengine/links.info.md");

    if pfad.exists() {
        // add the content to the links file
        let inhalt = fs::read_to_string(".samengine/links.info.md").unwrap();
        let _ = write!(file, "{}\n\n", inhalt);
    } else {
        println!("Info file doesnt exist!");
    }

    let _ = write!(file, "Used for {}: \n\n", config.projectname);

    for l in &config.links {
        let _ = write!(file, "- ");

        if let Some(name) = &l.name {
            let _ = write!(file, "**{}** ", name);
        }

        write!(file, " ([{}]({})) ", l.link, l.link).unwrap();

        let _ = write!(file, "- {} - ", l.description);

        if let Some(author) = &l.author {
            let _ = write!(file, "by **{}** ", author);
        }

        if let Some(lic) = &l.license {
            let _ = write!(file, "licensed unter *{}* ", lic);
        }

        if let Some(ll) = &l.licenselink {
            let _ = write!(file, "([{}]({})) ", ll, ll);
        }

        if l.changenotice {
            let _ = write!(file, "- *(changes were made)*");
        }

        let _ = write!(file, "\n");
    }

    for l in &config.links2 {
        let _ = write!(file, "- {}\n", l);
    }

    for path in &config.links3 {
        if Path::new(path).exists() {
            match fs::read_to_string(path) {
                Ok(content) => {
                    let _ = write!(file, "\n{}\n", content);
                }
                Err(e) => {
                    eprintln!("Warning: Could not read '{}': {}", path, e);
                }
            }
        } else {
            eprintln!("Warning: License file '{}' does not exist.", path);
        }
    }

    let _ = write!(
        file,
        "\n\n*File generated by linksaver from seg* - [samengine.vercel.app/packages](https://samengine.vercel.app/packages)\n"
    );

    println!(r#"Created File - Use parseMarkdown from samengine to make it into a nice html file.

npm i samengine
npx samengine markdown .samengine/links.md"#);
}


/// Function to convert the links json file into a txt file
fn viewx(config: &AppConfig) {
    let mut file = File::create(".samengine/links.txt").unwrap();

    write!(file, "Links for {}\n\n", config.projectname).unwrap();

    // Add Info Message
    let pfad = Path::new(".samengine/links.info.txt");

    if pfad.exists() {
        // add the content to the links file
        let inhalt = fs::read_to_string(".samengine/links.info.txt").unwrap();
        let _ = write!(file, "{}\n\n", inhalt);
    } else {
        println!("Info file doesnt exist!");
    }

    let _ = write!(file, "Used for {}: \n\n", config.projectname);

    for l in &config.links {
        if let Some(name) = &l.name {
            let _ = write!(file, "- {}", name);
        }

        write!(file, " ({}) ", l.link).unwrap();

        let _ = write!(file, "- {} - ", l.description);

        if let Some(author) = &l.author {
            let _ = write!(file, "by {} ", author);
        }

        if let Some(lic) = &l.license {
            let _ = write!(file, "licensed unter {} ", lic);
        }

        if let Some(ll) = &l.licenselink {
            let _ = write!(file, "({}) ", ll);
        }

        if l.changenotice {
            let _ = write!(file, "- (changes were made)");
        }

        let _ = write!(file, "\n");
    }

    for l in &config.links2 {
        let _ = write!(file, "- {}", l);
    }

    let _ = write!(
        file,
        "\nFile generated by linksaver from samtool - https://samengine.vercel.app/packages\n"
    );
}

fn list(config: &AppConfig) {
    println!("\nCredits:\n");

    for l in &config.links {
        if !l.showinlist {
            continue;
        }

        println!(
            "\"{}\" ({}) by {} is licensed under {} ({}){}",
            l.name.clone().unwrap_or_default(),
            l.link,
            l.author.clone().unwrap_or_default(),
            l.license.clone().unwrap_or_default(),
            l.licenselink.clone().unwrap_or_default(),
            if l.changenotice {
                " (changes were made)"
            } else {
                ""
            }
        );
    }

    for e in &config.links2 {
        println!("{}", e);
    }
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
        "add2" => add2(&mut config),
        "add3" => add3(&mut config),
        "view" => view(&config),
        "viewx" => viewx(&config),
        "list" => list(&config),
        _ => open_all(&config),
    }
}
