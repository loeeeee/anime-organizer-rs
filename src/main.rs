mod series;
mod tests;

use std::fs;

use anime_organizer_rs::load_env_var;
use dotenvy::dotenv;
use log::debug;
use log::error;
use log::info;
use log::warn;



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
    
    let paths = match fs::read_dir(&source_directory) {
        Ok(paths) => paths,
        Err(e) => {
            error!("Failed open {}, due to {}", &source_directory, &e);
            panic!();
        }
    };
}
