use std::fs;
use log::error;
use log::warn;
use log::info;

pub fn create_folder(full_path: &str) {
    if !fs::metadata(&full_path).is_ok() {
        fs::create_dir_all(&full_path).unwrap();
        info!("Created target folder successfully.")
    } else {
        info!("Target folder already exists. Skipping the creation.")
    }
}