#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use app::mutex_lock;
use nfd::Response;
use once_cell::sync::Lazy;
use std::collections::{BTreeMap, HashMap};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter};
use std::sync::Mutex;
use tauri::InvokeError;

static DICT: Lazy<Mutex<Option<BTreeMap<String, Vec<String>>>>> = Lazy::new(|| Mutex::new(None));
static HEADER: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
static INVERSE_DICT: Lazy<Mutex<Option<HashMap<String, Vec<String>>>>> =
    Lazy::new(|| Mutex::new(None));

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            pick_file,
            load_file,
            write_to_file,
            compose_code,
            add_word,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[tauri::command]
fn pick_file() -> Option<String> {
    let response = nfd::dialog().open().unwrap();
    match response {
        Response::Okay(f) => Some(f),
        Response::OkayMultiple(_) => None,
        Response::Cancel => None,
    }
}

#[tauri::command]
fn load_file(path: &str) -> Result<(), InvokeError> {
    let mut dict: BTreeMap<String, Vec<String>> = BTreeMap::new();

    let file = File::open(path).map_err(|_| "Open file")?;
    let mut lines = BufReader::new(file).lines().map(|x| x.unwrap());

    let mut header = String::new();
    for x in &mut lines {
        header.push_str(&x);
        header.push('\n');
        if x == "..." {
            break;
        }
    }

    for line in lines {
        let split = line.split('\t').collect::<Vec<_>>();
        if split.len() != 2 {
            continue;
        }
        let word = split[0];
        let code = split[1];
        dict.entry(String::from(code))
            .or_insert_with(Vec::new)
            .push(String::from(word));
    }

    let mut inverse: HashMap<String, Vec<String>> = HashMap::new();
    // construct the inverse map
    for (code, words) in &dict {
        for word in words {
            inverse
                .entry(word.clone())
                .or_insert_with(Vec::new)
                .push(code.clone());
        }
    }

    mutex_lock!(DICT).replace(dict);
    mutex_lock!(HEADER).replace(header);
    mutex_lock!(INVERSE_DICT).replace(inverse);

    Ok(())
}

#[tauri::command]
fn write_to_file(path: &str) {
    let file = OpenOptions::new()
        .truncate(true)
        .create(true)
        .write(true)
        .read(true)
        .open(path)
        .unwrap();
    let mut writer = BufWriter::new(file);

    let guard = mutex_lock!(HEADER);
    use std::io::Write;
    write!(&mut writer, "{}", guard.as_ref().unwrap()).unwrap();
    drop(guard);

    let guard = mutex_lock!(DICT);
    let dict = guard.as_ref().unwrap();
    for (code, words) in dict {
        for word in words {
            writeln!(&mut writer, "{}\t{}", word, code).unwrap();
        }
    }
    drop(guard);
}

#[tauri::command]
fn compose_code(word: &str) -> Option<String> {
    let guard = mutex_lock!(INVERSE_DICT);
    let inverse_map = guard.as_ref().unwrap();

    let look_up = |char: char| {
        let Some(codes) = inverse_map.get(&format!("{}", char))
            else { return None };
        let result = codes.iter().max_by(|a, b| a.len().cmp(&b.len()));
        result
            .map(|x| {
                if x.len() < 2 {
                    // Wubi code composition needs full code
                    None
                } else {
                    Some(x)
                }
            })
            .flatten()
    };
    let look_up_multiple = |chars: &[char]| {
        let mut codes = Vec::with_capacity(chars.len());
        for c in chars {
            let Some(code) = look_up(*c) else { return None };
            codes.push(code);
        }
        Some(codes)
    };

    let chars = word.chars().collect::<Vec<_>>();
    match chars.len() {
        1 => {
            // just return itself
            return look_up(chars[0]).map(|x| x.clone());
        }
        2 => {
            // take: xx-- xx--
            let Some(codes) = look_up_multiple(&[chars[0], chars[1]])
                else { return None };
            Some(format!(
                "{}{}",
                String::from_iter(codes[0].chars().take(2)),
                String::from_iter(codes[1].chars().take(2))
            ))
        }
        3 => {
            // take: x--- x--- xx--
            let Some(codes) = look_up_multiple(&[chars[0], chars[1], chars[2]])
                else { return None };
            Some(format!(
                "{}{}{}",
                codes[0].chars().next().unwrap(),
                codes[1].chars().next().unwrap(),
                String::from_iter(codes[2].chars().take(2))
            ))
        }
        x @ _ if x >= 4 => {
            // take: x--- x--- x--- ---- ... ---- x---
            let Some(codes) = look_up_multiple(&[
                chars[0],
                chars[1],
                chars[2],
                *chars.last().unwrap(),
            ]) else { return None };
            Some(String::from_iter(
                codes.iter().map(|x| x.chars().next().unwrap()),
            ))
        }
        _ => None,
    }
}

#[tauri::command]
fn add_word(code: &str, word: &str) -> Result<(), InvokeError> {
    let mut guard = mutex_lock!(DICT);
    let dict = guard.as_mut().unwrap();
    let words = dict.entry(String::from(code)).or_insert_with(Vec::new);
    if words.contains(&String::from(word)) {
        return Err("Already exists".into());
    } else {
        words.push(word.into());
    }
    Ok(())
}
