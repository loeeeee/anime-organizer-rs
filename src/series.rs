use core::panic;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
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
    let file_names_length = file_names.len() as u16; // TODO: make it safer
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

            // Extract context in the first pass
            let mut context = HashMap::<u16, u16>::new();
            for file_name in &file_names {
                match string_find_episode_number(&file_name) {
                    Ok(candidates) => {
                        for candidate in candidates {
                            context.entry(candidate).and_modify(|counter| *counter += 1).or_insert(1);
                        }
                    },
                    Err(_) => {
                        warn!("Failed to extract episode number from file name {}", &file_name);
                        continue;
                    }
                };
            }

            // Indexing context using Inverted document frequency
            let mut indexed_context = HashMap::<u16, f32>::new();
            let total_document_count: u16 = file_names_length;
            for (key, frequency) in context.iter() {
                let inverted_document_frequency = frequency.clone() as f32 / total_document_count.clone() as f32;
                indexed_context.insert(key.clone(), inverted_document_frequency);
            }
            
            // Find episode number based on reg and context
            let mut result: Vec<Episode> = Vec::new();
            for file_name in &file_names {
                match string_find_episode_number(&file_name) {
                    Ok(candidates) => {
                        let mut scoring: Vec<(f32, u16)> = Vec::new(); // Storing file_name related episode_number in new vector
                        for candidate in candidates {
                            let score = match indexed_context.get(&candidate) {
                                Some(sth) => *sth,
                                None => {
                                    error!("Inconsistent find episode number function.");
                                    continue
                                }
                            };
                            scoring.push((score, candidate));
                        }
                        scoring.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                        result.push(Episode{sequence: scoring[0].1,..Default::default()})
                    },
                    Err(_) => {
                        warn!("Failed to find any  episode number in {}", &file_name);
                        continue
                    },
                }
            }

            return Some(result);
        }
    }
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
fn string_remove_years(input: &str) -> Result<String, ()> {
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
fn string_remove_duplicate_spaces(input: &str) -> Result<String, ()> {
    // Test covered
    Ok(Regex::new(r"\s+").unwrap().replace_all(&input, " ").trim().to_string())
}

/// Remove roman numerals ranging from 1 to 13 from the string
fn string_remove_roman_number(input: &str) -> Result<String, ()> {
    Ok(Regex::new(r"(?i)\s+M{0,4}(CM|CD|D?C{0,3})(XC|XL|L?X{0,3})(IX|IV|V?I{0,3})$").unwrap().replace_all(&input, " ").to_string())
}

fn string_find_episode_number(file_name: &str) -> Result<Vec<u16>, ()> {
    let mut result = Vec::<u16>::new();

    let mut clean_name = {
        let mut middleware = string_remove_filtered(&file_name).unwrap();
        middleware = string_remove_years(&middleware).unwrap();
        middleware = string_remove_empty_brackets(&middleware).unwrap();
        middleware = string_remove_duplicate_spaces(&middleware).unwrap();
        middleware = string_remove_file_extension(&middleware);
        middleware.trim().to_string()
    };
    // Clean name should contain:
    // Episode number and episode name (may contain numbers).

    // Deal with roman numerals first
    for (_, [roman_numeral]) in Regex::new(r"(?i)\s+M{0,4}(CM|CD|D?C{0,3})(XC|XL|L?X{0,3})(IX|IV|V?I{0,3})$").unwrap().captures_iter(&clean_name).map(|c| c.extract()) {
        match roman_to_int(&roman_numeral) {
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
    for (_ ,[common_number]) in Regex::new(r#"[[[:ascii:]]\[\({](\d{1,2})[[[:ascii:]]\]\)}]"#).unwrap().captures_iter(&clean_name).map(|c| c.extract()) {
        debug!("{}", &common_number);
        // result.push(common_number);
        match common_number.parse::<u16>() {
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

/// Removes file extension name from string
fn string_remove_file_extension(input: &str) -> String {
    Regex::new(r"\.\w{2,4}$").unwrap().replace_all(&input, " ").trim().to_string()
}

#[cfg(test)]
mod tests {
    use core::panic;
    use std::{fs, io::Read};

    use anime_organizer_rs::load_env_var;
    use log::{warn, error, info};
    use crate::series::{Series, FilterWords};

    fn setup() {
        // Load env
        dotenvy::from_filename("test.env").unwrap();

        // Init logger
        // env_logger::init();
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn load_test_sheet(test_sheet_name: &str) -> String {
        let tests_series_names_path = load_env_var(&test_sheet_name).unwrap();
        info!("Series Name test sheet: {}", &tests_series_names_path);

        match fs::read_to_string(tests_series_names_path) {
            Ok(tests) => {
                info!("Successfully read the test sheet.");
                tests
            },
            Err(e) => {
                error!("Failed to load test sheet, due to {}.", &e);
                panic!();
            }
        }
    }
    
    #[test]
    fn series_name_extraction() {
        // Setup
        setup();
        
        // Load test sheet        
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize)]
        struct SeriesName {
            folder_name: String,
            series_name: String,
        }

        let test_sheet: Vec<SeriesName> = serde_json::from_str(&load_test_sheet(&"TEST_SERIES_NAME".to_string())).expect("JSON was not well-formatted");
        
        // Filter words
        let filter_words = FilterWords::load();

        // Run test
        for i in test_sheet.iter() {
            info!("{}: {}", &i.folder_name, &i.series_name);
            assert_eq!(super::extract_series_name(&i.folder_name, &filter_words).unwrap(), i.series_name.to_string());
        };
    }

    #[test]
    fn series_season_number_extraction() {
        // Setup
        setup();

        // Load test sheet
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize)]
        struct SeasonNumber {
            folder_name: String,
            season_number: i16,
        }

        let test_sheet: Vec<SeasonNumber> = serde_json::from_str(&load_test_sheet(&"TEST_SERIES_SEASON_NUMBER".to_string())).expect("JSON was not well-formatted");

        // Filter words
        let filter_words = FilterWords::load();

        // Run test
        for i in test_sheet.iter() {
            info!("{}: {}", &i.folder_name, &i.season_number);
            assert_eq!(super::extract_series_season_number(&i.folder_name, &filter_words).unwrap().to_string(), i.season_number.to_string());
        };
    }

    #[test]
    fn string_duplication_space_removal() {
        // Setup
        setup();

        // Load test sheet
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize)]
        struct SpacedString {
            raw: String,
            result: String,
        }

        let test_sheet: Vec<SpacedString> = serde_json::from_str(&load_test_sheet(&"TEST_STRING_SPACE_DEDUPLICATION".to_string())).expect("JSON was not well-formatted");

        // Run test
        use super::string_remove_duplicate_spaces;
        for i in test_sheet.iter() {
            info!("{}: {}", &i.raw, &i.result);
            assert_eq!(string_remove_duplicate_spaces(&i.raw).unwrap(), i.result);
        }
    }

    #[test]
    fn string_year_removal() {
        // Setup
        setup();

        // Load test sheet
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize)]
        struct SpacedString {
            raw: String,
            result: String,
        }

        let test_sheet: Vec<SpacedString> = serde_json::from_str(&load_test_sheet(&"TEST_STRING_YEAR_REMOVAL".to_string())).expect("JSON was not well-formatted");

        // Run test
        use super::string_remove_years;
        for i in test_sheet.iter() {
            info!("{}: {}", &i.raw, &i.result);
            assert_eq!(string_remove_years(&i.raw).unwrap(), i.result);
        }
    }

    #[test]
    fn string_episode_number_discovery() {
        // Setup
        setup();

        // Load test sheet
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize)]
        struct EpisodeNumber {
            file_name: String,
            episode_number: u16,
        }

        let test_sheet: Vec<EpisodeNumber> = serde_json::from_str(&load_test_sheet(&"TEST_STRING_EPISODE_NUMBER_DISCOVERY".to_string())).expect("JSON was not well-formatted");

        // Run test
        use super::string_find_episode_number;
        for i in test_sheet.iter() {
            info!("{}: {}", &i.file_name, &i.episode_number);
            assert_eq!(string_find_episode_number(&i.file_name).unwrap()[0], i.episode_number);
        }
    }

    #[test]
    fn episode_number_extraction_with_context() {
        // Setup
        setup();

        // Load test sheet
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize)]
        struct EpisodeNumber {
            file_names: Vec<String>,
            episode_numbers: Vec<u16>,
        }

        let test_sheet: Vec<EpisodeNumber> = serde_json::from_str(&load_test_sheet(&"TEST_EPISODE_NUMBER_EXTRACTION_WITH_CONTEXT".to_string())).expect("JSON was not well-formatted");

        // Run test
        use super::extract_episode_number;
        for i in test_sheet {
            // Find episode number
            let result = extract_episode_number(i.file_names).unwrap();

            // Assert result
            let assert_iter = result.iter().zip(i.episode_numbers.iter()); // Zip two chile elements of episode number together
            for (prediction, ground_truth) in assert_iter {
                info!("{}: {}", &prediction.sequence, &ground_truth);
                assert_eq!(prediction.sequence, *ground_truth);
            }
        }
    }
}