use std::fs;
use log::error;
use log::warn;
use log::info;
use regex::Regex;

use crate::FilterWords;

pub fn create_folder(full_path: &str) {
    if !fs::metadata(&full_path).is_ok() {
        fs::create_dir_all(&full_path).unwrap();
        info!("Created target folder successfully.")
    } else {
        info!("Target folder already exists. Skipping the creation.")
    }
}

struct Series {
    name: String,
    location: String,
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

    fn extract_series_name(&self, folder_name: &str, filter_words: &FilterWords) -> Result<String, ()> {
        /// # Return the cleaned series name inferred from folder name
    
        // Remove CC group name
        let filter_construct_middleware: Vec<String> = FilterWords::cc_group.iter()
        .map(|i| format!("({})", i))
        .collect();
    
        let combined = filter_construct_middleware.join("|").replace(".", ".").replace("-", "-");
        let reg_str = format!(r"{}(&{})*?", combined, combined);
        let reg = Regex::new(&reg_str).expect("Invalid regex pattern");
    
    
    
        // Remove meta tags
        todo!() 
    }
}


fn extract_episode_number(file_name: &str) -> Result<String, ()> {
    todo!()
}