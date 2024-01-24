use core::panic;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use log::debug;
use log::error;
use log::warn;
use log::info;
use regex::Regex;
use serde::Deserialize;
use std::result::Result;

#[derive(Debug, Deserialize)]
pub struct FilterWords {
    cc_group: Vec<String>,
    meta_tag: Vec<String>,
    low_priority: Vec<String>,
}

impl FilterWords {
    pub fn load() -> FilterWords {
        // TODO: Cache Load function
        let filter_words_file_path = "./static/filter_words.yaml";
        let mut filter_words_string = String::new();
        match File::open(filter_words_file_path).unwrap().read_to_string(&mut filter_words_string) {
            Ok(_) => info!("Load filter words successfully."),
            Err(_) => error!("Failed to load filter words, please check {}", &filter_words_file_path),
        };
    
        // Parse the YAML content into the ServerConfig struct
        match serde_yaml::from_str(&filter_words_string) {
            Ok(content) => {
                info!("Parse filter words successfully.");
                content
            },
            Err(err) => {
                error!("Error parsing YAML: {}", err);
                panic!();
            }
        }
    }
}


pub fn create_folder(full_path: &str) {
    if !fs::metadata(&full_path).is_ok() {
        fs::create_dir_all(&full_path).unwrap();
        info!("Created target folder successfully.")
    } else {
        info!("Target folder already exists. Skipping the creation.")
    }
}

pub struct Series {
    name: String,
    location: PathBuf,
    seasons: Vec<Season>,
}

struct Season {
    sequence: i8,
    location: String,
    episodes: Vec<Episode>,
}

struct Episode {
    sequence: i16,
    location: String,
    subtitles: Vec<Subtitle>,
}

struct Subtitle {
    language: String,
    location: String,
}

impl Series {
    pub fn new(folder_path: &str) -> Result<Series, ()> {
        // Entry point for Series struct
        debug!("Folder path: {}", &folder_path);

        let filter_words = FilterWords::load();

        let mut series = Series {
            name: "".to_string(),
            location: PathBuf::from(&folder_path),
            seasons: Vec::new(),
        };

        let folder_name = match series.location.file_name() {
            Some(name) => {
                debug!("Folder name: {}", &name.to_string_lossy());
                name
            },
            None => {
                warn!("Failed to get folder name.");
                panic!("")
            }
        };

        series.name = match extract_series_name(&folder_name.to_string_lossy(), &filter_words) {
            Ok(name) => {
                info!("Series name: {}", &name);
                name
            },
            Err(e) => {
                warn!("Failed to extract series name, due to {}", &e.to_string());
                panic!("")
            }
        };

        todo!()
    }
}

fn basic_file_name_cleaning(file_name: &str, filter_words: &FilterWords) -> Result<String, ()> { // TODO: Make this a static method
    // Remove CC group name
    let mut result = {
        let filter_construct_middleware: Vec<String> = filter_words.cc_group.iter()
            .map(|i| format!("({})", i))
            .collect();
    
        let combined = filter_construct_middleware.join("|");
        let reg_str = format!(r"(?i){}(&{})*?", combined, combined);
        let reg = Regex::new(&reg_str).expect("Invalid regex pattern");
        reg.replace_all(&file_name, "%ReM0vE%").to_string()
    };

    // Remove meta tags
    result = {
        let filter_construct_middleware: Vec<String> = filter_words.meta_tag.iter()
            .map(|i| format!("({})", i))
            .collect();
    
        let combined = filter_construct_middleware.join("|");
        let reg_str = format!(r"(?i){}(&{})*?", combined, combined);
        let reg = Regex::new(&reg_str).expect("Invalid regex pattern");
        reg.replace_all(&result, "%ReM0vE%").to_string()
    };

    // Remove %ReM0vE%
    result = {
        let reg = Regex::new(r"\[[^\]]*?(%ReM0vE%)[^\[]*?\]").unwrap();
        reg.replace_all(&result, "").to_string()
    };
    result = {
        let reg = Regex::new(r"\([^\]]*?(%ReM0vE%)[^\[]*?\)").unwrap();
        reg.replace_all(&result, "").to_string()
    };
    debug!("After removing %ReM0vE%: {}", &result);

    // Remove square brackets
    result = {
        let reg = Regex::new(r"\[\W*?\]").unwrap();
        reg.replace_all(&result, "").to_string()
    };

    debug!("After removing square brackets: {}", &result);

    // Remove random things
    result = {
        let reg = Regex::new(r"\[[(\s)-_]*\]").unwrap();
        reg.replace_all(&result, "").to_string()
    };
    
    debug!("After removing random things: {}", &result);
    
    // Trim
    Ok(result.trim().to_string())
}

fn extract_episode_number(file_name: &str) -> Result<i16, ()> {
    let mut clean_name = basic_file_name_cleaning(&file_name, &FilterWords::load()).unwrap(); // TODO: Use cache
    
    // Remove special characters
    clean_name = string_remove_symbols(&clean_name).unwrap();

    // Remove ENG Chars
    clean_name = Regex::new(r"[A-Za-z]").unwrap().replace_all(&clean_name, " ").trim().to_string();

    // Remove year numbers
    clean_name = string_remove_years(&clean_name).unwrap();

    // Get EP number, assuming ep number is [0,100)
    match Regex::new(r"\d{1,2}").unwrap().find(&clean_name).unwrap().as_str().parse::<i16>() {
        Ok(ep_number) => {
            debug!("Find episode number {}", &ep_number);
            Ok(ep_number)
        },
        Err(e) => {
            warn!("Failed to find episode number");
            Err(e)
        },
    }
}

pub fn extract_series_name(folder_name: &str, filter_words: &FilterWords) -> Result<String, ()> {

    let mut result = basic_file_name_cleaning(&folder_name, &filter_words).unwrap(); // TODO: Use cache in struct

    // Remove Roman numbers
    result = {
        let reg = Regex::new(r"(?i)\s+(I{1,3}|IV|VI{0,3}|IX|XI{0,3})$").unwrap();
        reg.replace_all(&result, "").to_string()
    };

    debug!("After removing roman numbers: {}", &result);

    Ok(result.trim().to_string())
}

pub fn extract_series_season_number(file_name: &str, filter_words: &FilterWords) -> Result<i16, ()> { // TODO: Move this function to struct

    let clean_file_name = basic_file_name_cleaning(&file_name, &filter_words).unwrap();

    // Extract from Roman numerals
    {
        let reg = Regex::new(r"(?i)\s+(I{1,3}|IV|VI{0,3}|IX|XI{0,3})$").unwrap();
        match reg.captures(&clean_file_name) {
            Some(caps) => match roman_to_int(&caps[1]).try_into() {
                Ok(season_number) => {
                    debug!("Successfully extract season number from Roman numeral, {}", &season_number);
                    return Ok(season_number);
                },
                Err(_) => debug!("Fail to infer season number from Roman numeral."),
            },
            None => debug!("Fail to infer season number from Roman numeral."),
        }
    }

    // Extract from explicit season number
    {
        let reg = Regex::new(r"(?i)\s+(?:season|S)\s*(\d+)$").unwrap();
        match reg.captures(&clean_file_name) {
            Some(caps) => match &caps[1].parse::<i16>() {
                Ok(season_number) => {
                    debug!("Successfully extract season number from explicit season number, {}", &season_number);
                    return Ok(*season_number);
                },
                Err(_) => debug!("Fail to infer season number from explicit season number."),
            },
            None => debug!("Fail to infer season number from explicit season number."),
        }
    }

    // TODO: Non-explicit season number
    Ok(1)
}

fn roman_to_int(roman: &str) -> i32 {
    // Convert roman numeral to integer
    let mut result = 0;
    let mut prev_value = 0;

    for c in roman.chars().rev() {
        let value = match c {
            'I' => 1,
            'V' => 5,
            'X' => 10,
            'L' => 50,
            'C' => 100,
            'D' => 500,
            'M' => 1000,
            _ => panic!("Invalid Roman numeral"),
        };

        if value < prev_value {
            result -= value;
        } else {
            result += value;
        }

        prev_value = value;
    }

    result
}

enum FileExtensionNames {

}

fn extract_file_extension(file_name: &str) -> FileExtensionNames {
    todo!()
}

fn string_remove_symbols(input: &str) -> Result<String, ()> {
    // Removes all special characters in a string
    // let test = Regex::new(r#"""#).unwrap();
    Ok(Regex::new(r#"[!@#$%^&*()_+{}\[\]:;"'<>,.?\|`~=-\\]"#).unwrap().replace_all(&input, "").to_string())
}

pub fn string_remove_years(input: &str) -> Result<String, ()> {
    let reg = Regex::new(r"\d{4}").unwrap();
    let candidates = reg.find(&input);
    let mut result = input.to_string();
    for year_candidate in candidates  {
        let year = year_candidate.as_str().parse::<i32>().unwrap();
        if year >= 1928 && year <= 2030 {
            let reg = Regex::new(&year.to_string()).unwrap();
            result = reg.replace(&result, " ").to_string();
        }
    };
    Ok(result.trim().to_string())
}

pub fn string_remove_duplicate_spaces(input: &str) -> Result<String, ()> {
    Ok(Regex::new(r"\s+").unwrap().replace_all(&input, " ").trim().to_string())
}