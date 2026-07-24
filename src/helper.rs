use std::{collections::HashMap, fs::File, io::Read, path::Path};

fn read_env_vars() -> HashMap<String, String> {
    let mut map = HashMap::new();

    let path = Path::new(".env");

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", path.display(), why),
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", path.display(), why),
        Ok(_) => (),
    }

    for line in s.lines() {
        if line.starts_with('#') {
            continue;
        }
        if let Some((s1, s2)) = line.split_once('=') {
            map.insert(s1.trim().to_string(), s2.trim().to_string());
        }
    }

    map
}

pub fn get_lichess_token() -> String {
    let key = "LICHESS_TOKEN";
    let map = read_env_vars();
    match map.get(key) {
        None => panic!("No LICHESS_TOKEN found in \".env\""),
        Some(value) => return value.clone(),
    }
}
