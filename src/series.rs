use core::panic;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::hash::Hash;
use std::io::Read;
use std::collections::HashSet;
use std::num::ParseIntError;
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

pub struct Episode {
    sequence: u16,
    location: String,
    subtitles: Vec<Subtitle>,
}

impl Default for Episode {
    fn default() -> Self {
        Self {
            sequence: 0,
            location: "".to_string(),
            subtitles: Vec::new()
        }
    }
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
                warn!("Failed to extract series name");
                panic!("")
            }
        };

        todo!()
    }
}

pub fn extract_series_name(folder_name: &str, filter_words: &FilterWords) -> Result<String, ()> {
    // Test covered
    // let mut result = string_remove_square_brackets(&string_remove_filtered(&folder_name).unwrap()).unwrap().trim().to_string();// basic_file_name_cleaning(&folder_name, &filter_words).unwrap(); // TODO: Use cache in struct
    let mut result = {
        let mut middleware = string_remove_filtered(&folder_name).unwrap();
        middleware = string_remove_years(&middleware).unwrap();
        middleware = string_remove_roman_number(&middleware).unwrap();
        middleware = string_remove_episode_range(&middleware).unwrap();
        middleware = string_remove_empty_brackets(&middleware).unwrap();
        middleware = string_remove_duplicate_spaces(&middleware).unwrap();
        middleware.trim().to_string()
    };

    // Remove Roman numbers
    result = string_remove_roman_number(&result).unwrap(); 

    debug!("After removing roman numbers: {}", &result);

    Ok(result.trim().to_string())
}

pub fn extract_series_season_number(file_name: &str, filter_words: &FilterWords) -> Result<u16, ()> { // TODO: Move this function to struct
    // Test covered
    let clean_file_name = string_remove_square_brackets(&string_remove_filtered(&file_name).unwrap()).unwrap().trim().to_string();

    // Extract from Roman numerals
    {
        let reg = Regex::new(r"(?i)\s+(I{1,3}|IV|VI{0,3}|IX|XI{0,3})$").unwrap();
        match reg.captures(&clean_file_name) {
            Some(caps) => match roman_to_int(&caps[1]) {
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
            Some(caps) => match &caps[1].parse::<u16>() {
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

enum EpisodeType {
    Main(u16),
    Extra(u16)    
}

fn extract_episode_type() {
    todo!()
}

pub fn extract_episode_number(file_names: Vec<String>) -> Option<Vec<Episode>> {
    // This function deals with a list of file names (0 to inf), build context to find the unique numbers in the list of files,
    // and return a list of files with their episode number.
    match file_names.len() {
        0 => {
            return None;
        },
        1 => {
            // Extract episode number without context
            // Make the assumption that the correct one is the first one
            // TODO: Improve algorithm
            match string_find_episode_number(&file_names[0]) {
                Ok(ep_number) => {
                    debug!("Find episode number {}", &ep_number[0]);
                    return Some(vec![Episode{sequence: ep_number[0], ..Default::default() }]);
                },
                Err(_) => {
                    warn!("Failed to find episode number.");
                    return None
                }
            }
        },
        _ => {
            // Extract episode number with context
            let mut context = HashSet::<String>::new();
            for file_name in file_names {

            }

            todo!()
        }
    }

    todo!()
}

// Extract helper

/// Convert roman numeral to integer
fn roman_to_int(roman: &str) -> Result<u16, ()> {
    let mut result: i32 = 0;
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
            _ => {
                warn!("Failed to parse number in roman numeral format");
                return Err(());
            },
        };

        if value < prev_value {
            result -= value;
        } else {
            result += value;
        }

        prev_value = value;
    }

    Ok(result.try_into().unwrap())
}

enum FileExtensionNames {

}

fn extract_file_extension(file_name: &str) -> FileExtensionNames {
    todo!()
}

// String helper

/// Removes all special characters in a string
fn string_remove_special_characters(input: &str) -> Result<String, ()> {
    Ok(Regex::new(r#"[!@#$%^&*()_+{}\[\]:;"'<>,.?\|`~=-\\]"#).unwrap().replace_all(&input, " ").to_string())
}

/// Removes year numbers from string ranging from 1928 to 2030
pub fn string_remove_years(input: &str) -> Result<String, ()> {
    // Test covered
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

    // Remove duplicated spaces
    result = string_remove_duplicate_spaces(&result).unwrap();

    Ok(result.trim().to_string())
}

/// Removes duplicate spaces in the string
pub fn string_remove_duplicate_spaces(input: &str) -> Result<String, ()> {
    // Test covered
    Ok(Regex::new(r"\s+").unwrap().replace_all(&input, " ").trim().to_string())
}

/// Remove roman numerals ranging from 1 to 13 from the string
fn string_remove_roman_number(input: &str) -> Result<String, ()> {
    Ok(Regex::new(r"(?i)\s+M{0,4}(CM|CD|D?C{0,3})(XC|XL|L?X{0,3})(IX|IV|V?I{0,3})$").unwrap().replace_all(&input, " ").to_string())
}

pub fn string_find_episode_number(file_name: &str) -> Result<Vec<u16>, ParseIntError> {
    let mut result = Vec::<u16>::new();

    let mut clean_name = {
        let mut middleware = string_remove_filtered(&file_name).unwrap();
        middleware = string_remove_years(&middleware).unwrap();
        middleware = string_remove_empty_brackets(&middleware).unwrap();
        middleware = string_remove_duplicate_spaces(&middleware).unwrap();
        middleware.trim().to_string()
    };
    // Clean name should contain:
    // Episode number and episode name (may contain numbers).

    // Deal with roman numerals first
    for roman_numeral in Regex::new(r"(?i)\s+M{0,4}(CM|CD|D?C{0,3})(XC|XL|L?X{0,3})(IX|IV|V?I{0,3})$").unwrap().find(&clean_name) {
        match roman_to_int(&roman_numeral.as_str()) {
            Ok(episode_number_guess) => {
                debug!("Find episode number candidates {}", episode_number_guess);
                result.push(episode_number_guess);
            },
            Err(_) => {
                warn!("Failed to parse roman numerals");
            },
        }
    }

    // Deal with common numbers
    for common_number in Regex::new(r"\d{1,2}").unwrap().find(&clean_name) {
        match common_number.as_str().parse::<u16>() {
            Ok(success) => {
                debug!("Find episode number candidate {}", success);
                result.push(success)
            },
            Err(_) => {
                warn!("Failed to parse common numbers");
            },
        }
    }

    Ok(result)
}

/// Remove CC names and Meta tags in given string
fn string_remove_filtered(input: &str) -> Result<String, ()> {
    // Remove CC names
    let filter_words = FilterWords::load();
    let mut result = {
        let filter_construct_middleware: Vec<String> = filter_words.cc_group.iter()
            .map(|i| format!("({})", i))
            .collect();
    
        let combined = filter_construct_middleware.join("|");
        let reg_str = format!(r"(?i){}(&{})*?", combined, combined);
        let reg = Regex::new(&reg_str).expect("Invalid regex pattern");
        reg.replace_all(&input, "%ReM0vE%").to_string()
    };

    // Remove Meta Tags
    result = {
        let filter_construct_middleware: Vec<String> = filter_words.meta_tag.iter()
            .map(|i| format!("({})", i))
            .collect();
    
        let combined = filter_construct_middleware.join("|");
        let reg_str = format!(r"(?i){}(&{})*?", combined, combined);
        let reg = Regex::new(&reg_str).expect("Invalid regex pattern");
        reg.replace_all(&result, "%ReM0vE%").to_string()
    };

    // Remove REMOVE
    result = {
        let reg = Regex::new(r"\[[^\]]*?(%ReM0vE%)[^\[]*?\]").unwrap();
        reg.replace_all(&result, " ").to_string()
    };
    result = {
        let reg = Regex::new(r"\([^\]]*?(%ReM0vE%)[^\[]*?\)").unwrap();
        reg.replace_all(&result, " ").to_string()
    };

    debug!("After removing filtered words: {}", &result);

    Ok(result)
}

/// Remove square brackets with content inside (Brutal)
fn string_remove_square_brackets(input: &str) -> Result<String, ()> {
     // TODO: Not using this
    Ok(Regex::new(r"\[\W*?\]").unwrap().replace_all(&input, " ").to_string())
}

/// Remove things like [01-13]
fn string_remove_episode_range(input: &str) -> Result<String, ()> {
    let mut result = Regex::new(r"\d{1,3}-\d{1,3}").unwrap().replace_all(&input, " ").to_string();

    // Remove empty brackets
    result = string_remove_empty_brackets(&result).unwrap();

    Ok(result)
}

/// Remove empty brackets like [ ] ( ) {  }
/// Naive algorithm is used
fn string_remove_empty_brackets(input: &str) -> Result<String, ()> {
    // TODO: Change algorithm to allow nested empty brackets
    Ok(Regex::new(r"[\[\({})]\s*?[\]\)}]").unwrap().replace_all(&input, " ").to_string())
}

/// Removes english characters from string
fn string_remove_english_characters(input: &str) -> Result<String, ()> {
    Ok(Regex::new(r"[A-Za-z]").unwrap().replace_all(&input, " ").trim().to_string())
}