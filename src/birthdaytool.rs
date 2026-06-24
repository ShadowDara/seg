use chrono::{Datelike, Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Birthday {
    name: String,
    date: String, // YYYY-MM-DD
}

fn get_file_path(args: &[String]) -> PathBuf {
    args.windows(2)
        .find(|w| w[0] == "--file")
        .map(|w| PathBuf::from(&w[1]))
        .unwrap_or_else(|| PathBuf::from("../Secret2/birthdays.json"))
}

// ---------- Storage ----------

fn load_birthdays(path: &PathBuf) -> Vec<Birthday> {
    match fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn save_birthdays(path: &PathBuf, data: &[Birthday]) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let json = serde_json::to_string_pretty(data).unwrap();
    fs::write(path, json).expect("Konnte birthdays.json nicht schreiben");
}

// ---------- Date Helpers ----------

fn get_next_occurrence(date_str: &str) -> Option<NaiveDate> {
    let birthday = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()?;
    let today = Local::now().date_naive();

    let mut next = NaiveDate::from_ymd_opt(
        today.year(),
        birthday.month(),
        birthday.day(),
    )?;

    if next < today {
        next = NaiveDate::from_ymd_opt(
            today.year() + 1,
            birthday.month(),
            birthday.day(),
        )?;
    }

    Some(next)
}

fn is_same_day(date_str: &str, today: NaiveDate) -> bool {
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        date.day() == today.day() && date.month() == today.month()
    } else {
        false
    }
}

// ---------- Commands ----------

fn add(path: &PathBuf, name: String, date: String) {
    let mut data = load_birthdays(path);

    data.push(Birthday {
        name: name.clone(),
        date,
    });

    save_birthdays(path, &data);

    println!("✅ {} gespeichert", name);
}

fn list(path: &PathBuf) {
    let data = load_birthdays(path);

    if data.is_empty() {
        println!("📭 Leer");
        return;
    }

    for (i, b) in data.iter().enumerate() {
        println!("{}. {} - {}", i + 1, b.name, b.date);
    }
}

fn today_cmd(path: &PathBuf) {
    let data = load_birthdays(path);
    let today = Local::now().date_naive();

    let birthdays: Vec<_> = data
        .iter()
        .filter(|b| is_same_day(&b.date, today))
        .collect();

    println!("🎉 Heute:");

    if birthdays.is_empty() {
        println!("Niemand 😢");
    } else {
        for b in birthdays {
            println!("{}", b.name);
        }
    }
}

fn week(path: &PathBuf) {
    let data = load_birthdays(path);

    let today = Local::now().date_naive();
    let in_7_days = today + Duration::days(7);

    let mut birthdays: Vec<_> = data
        .into_iter()
        .filter_map(|b| {
            let next = get_next_occurrence(&b.date)?;

            if next >= today && next <= in_7_days {
                Some((b, next))
            } else {
                None
            }
        })
        .collect();

    birthdays.sort_by_key(|(_, next)| *next);

    println!("📅 Nächste 7 Tage:");

    if birthdays.is_empty() {
        println!("Niemand 😢");
    } else {
        for (b, next) in birthdays {
            println!("{} - {}", b.name, next);
        }
    }
}

fn month(path: &PathBuf) {
    let data = load_birthdays(path);
    let current_month = Local::now().month();

    let birthdays: Vec<_> = data
        .iter()
        .filter(|b| {
            NaiveDate::parse_from_str(&b.date, "%Y-%m-%d")
                .map(|d| d.month() == current_month)
                .unwrap_or(false)
        })
        .collect();

    println!("🗓️ Dieser Monat:");

    if birthdays.is_empty() {
        println!("Niemand 😢");
    } else {
        for b in birthdays {
            println!("{} - {}", b.name, b.date);
        }
    }
}

fn upcoming(path: &PathBuf) {
    let data = load_birthdays(path);

    let mut birthdays: Vec<_> = data
        .into_iter()
        .filter_map(|b| {
            let next = get_next_occurrence(&b.date)?;
            Some((b, next))
        })
        .collect();

    birthdays.sort_by_key(|(_, next)| *next);

    println!("⏳ Kommend:");

    if birthdays.is_empty() {
        println!("Niemand 😢");
    } else {
        for (b, next) in birthdays {
            println!("{} - {}", b.name, next);
        }
    }
}

// ---------- CLI ----------

fn print_help() {
    println!(
        r#"
🎂 Birthday CLI

Commands:
  add <name> <YYYY-MM-DD>
  list
  today
  week
  month
  upcoming

Example:
  cargo run -- add Max 1990-05-20
"#
    );
}

pub fn bmain() {
    let args: Vec<String> = std::env::args().collect();

    let path = get_file_path(&args);

    let Some(cmd) = args.get(2).map(String::as_str) else {
        print_help();
        return;
    };

    match cmd {
        "add" => {
            let Some(name) = args.get(3) else {
                print_help();
                return;
            };

            let Some(date) = args.get(4) else {
                print_help();
                return;
            };

            add(&path, name.clone(), date.clone());
        }

        "list" => list(&path),
        "today" => today_cmd(&path),
        "week" => week(&path),
        "month" => month(&path),
        "upcoming" => upcoming(&path),

        _ => print_help(),
    }
}