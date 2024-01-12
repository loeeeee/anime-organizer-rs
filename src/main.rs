mod series;

use std::fs::File;
use std::io::Read;

use env_logger::filter::Filter;
use log::debug;
use log::error;
use log::info;
use log::warn;
use dotenv::dotenv;
use serde::Deserialize;


fn load_env_var(var_name: &str) -> Result<String, std::env::VarError> {
    debug!("Loading env var: {}", &var_name);
    match std::env::var(&var_name) {
        Ok(var_value) => {
            info!("{}={}", &var_name, &var_value);
            return Ok(var_value)
        },
        Err(e) => {
            error!("Failed to load TARGET_DIR env var. Please check .env file.");
            return Err(e)
        },
    };
}

#[derive(Debug, Deserialize)]
struct FilterWords {
    cc_group: Vec<String>,
    meta_tag: Vec<String>,
    low_priority: Vec<String>,
}

impl FilterWords {
    fn load() -> FilterWords {
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

fn main() {
    // Load .env file
    dotenv().ok(); // Need to be loaded before logger init for setting env vars up.

    // Init logger
    env_logger::init();

    // TODO: Check source directory
    let source_directory = load_env_var(&"SOURCE_DIR").unwrap();

    // Check and create target directory
    debug!("Checking and creating folder for target mapping.");
    let target_directory = load_env_var(&"TARGET_DIR").unwrap();
    series::create_folder(&target_directory);
    debug!("Finish creating folder {} for target mapping.", target_directory);

    // Iterate through source directory to resolve each anime series
    
}
