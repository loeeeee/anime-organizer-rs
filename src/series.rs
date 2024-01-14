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

#[derive(Debug, Deserialize)]
pub struct FilterWords {
    cc_group: Vec<String>,
    meta_tag: Vec<String>,
    low_priority: Vec<String>,
}

impl FilterWords {
    pub fn load() -> FilterWords {
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

        // let series_name = match Self::extract_series_name {
        //     Ok(result) => {
        //         info!("Find series {}", &result);
        //         result
        //     },
        //     Err(e) => {
        //         warn!("Failed to extract series name from {}, due to {}", &folder_path, &e);
        //         None
        //     }
        // };
        // if series_name == None {
        //     return Err(())
        // }

        todo!()
    }


}


fn extract_episode_number(file_name: &str) -> Result<String, ()> {
    todo!()
}

pub fn extract_series_name(folder_name: &str, filter_words: &FilterWords) -> Result<String, std::io::Error> {
    /// # Return the cleaned series name inferred from folder name

    // Remove CC group name
    let mut result = {
        let filter_construct_middleware: Vec<String> = filter_words.cc_group.iter()
            .map(|i| format!("({})", i))
            .collect();
    
        let combined = filter_construct_middleware.join("|");
        let reg_str = format!(r"{}(&{})*?", combined, combined);
        let reg = Regex::new(&reg_str).expect("Invalid regex pattern"); // Todo: Add ignore case
        reg.replace_all(&folder_name, "%ReM0vE%").to_string()
    };

    // Remove meta tags
    result = {
        let filter_construct_middleware: Vec<String> = filter_words.meta_tag.iter()
            .map(|i| format!("({})", i))
            .collect();
    
        let combined = filter_construct_middleware.join("|");
        let reg_str = format!(r"{}(&{})*?", combined, combined);
        let reg = Regex::new(&reg_str).expect("Invalid regex pattern"); // Todo: Add ignore case
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

    // debug!("{}", &reg);

    Ok(result.trim().to_string())
    
}