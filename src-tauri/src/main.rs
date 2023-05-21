#![feature(try_blocks)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::collections::{BTreeMap, HashMap};
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufRead, BufReader, BufWriter};
use std::sync::Mutex;

use nfd::Response;
use once_cell::sync::Lazy;
use tauri::InvokeError;

use app::mutex_lock;

type StaticType<T> = Lazy<Mutex<Option<T>>>;
macro_rules! static_type_initializer {
    () => {
        Lazy::new(|| Mutex::new(None))
    };
}

static DICT: StaticType<BTreeMap<String, Vec<String>>> = static_type_initializer!();
static HEADER: StaticType<String> = static_type_initializer!();
static CHAR_MAP: StaticType<HashMap<char, String>> = static_type_initializer!();

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            pick_file,
            load_file,
            write_to_file,
            compose_code,
            add_word,
            query_words,
            update_words,
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
fn load_file(dict_path: &str, char_map_path: &str) -> Result<(), InvokeError> {
    let result: anyhow::Result<()> = try {
        let mut dict: BTreeMap<String, Vec<String>> = BTreeMap::new();

        let file = File::open(dict_path)?;
        let lines = BufReader::new(file)
            .lines()
            .collect::<io::Result<Vec<String>>>()?;
        let mut header = String::new();
        for x in &lines {
            header.push_str(x);
            header.push('\n');
            if x == "..." {
                break;
            }
        }
        for line in &lines {
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
        mutex_lock!(DICT).replace(dict);
        mutex_lock!(HEADER).replace(header);

        let mut char_map = HashMap::new();
        let file = BufReader::new(File::open(char_map_path)?);
        for line in file.lines() {
            let line = line?;
            let split = line.split_whitespace().collect::<Vec<_>>();
            if split.len() != 2 {
                continue;
            }
            if split[0].chars().count() != 1 {
                continue;
            }
            let char = split[0].chars().next().unwrap();
            let code = split[1];
            char_map.insert(char, String::from(code));
        }
        mutex_lock!(CHAR_MAP).replace(char_map);
    };

    if let Err(e) = result {
        return Err(format!("{}", e).into());
    }

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
    let guard = mutex_lock!(CHAR_MAP);
    let char_map = guard.as_ref().unwrap();

    let look_up = |chars: &[char]| {
        let mut codes = Vec::with_capacity(chars.len());
        for c in chars {
            let Some(code) = char_map.get(c) else { return None };
            codes.push(code);
        }
        Some(codes)
    };

    let chars = word.chars().collect::<Vec<_>>();
    // not compose for single-char
    match chars.len() {
        2 => {
            // take: xx-- xx--
            let Some(codes) = look_up(&[chars[0], chars[1]])
                else { return None };
            Some(format!(
                "{}{}",
                String::from_iter(codes[0].chars().take(2)),
                String::from_iter(codes[1].chars().take(2))
            ))
        }
        3 => {
            // take: x--- x--- xx--
            let Some(codes) = look_up(&[chars[0], chars[1], chars[2]])
                else { return None };
            Some(format!(
                "{}{}{}",
                codes[0].chars().next().unwrap(),
                codes[1].chars().next().unwrap(),
                String::from_iter(codes[2].chars().take(2))
            ))
        }
        x if x >= 4 => {
            // take: x--- x--- x--- ---- ... ---- x---
            let Some(codes) = look_up(&[
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

#[tauri::command]
fn query_words(code: &str) -> Vec<String> {
    let mut guard = mutex_lock!(DICT);
    let dict = guard.as_mut().unwrap();
    dict.get(code).cloned().unwrap_or_default()
}

#[tauri::command]
fn update_words(code: &str, words: Vec<&str>) {
    let mut guard = mutex_lock!(DICT);
    let dict = guard.as_mut().unwrap();
    if let Some(v) = dict.get_mut(code) {
        *v = Vec::from_iter(words.iter().map(|x| String::from(*x)))
    }
}
