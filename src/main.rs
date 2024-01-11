mod series;

use log::debug;
use log::error;
use log::info;
use log::warn;
use dotenv::dotenv;

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
