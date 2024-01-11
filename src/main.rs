mod series;
use log::debug;
use log::error;
use log::info;
use log::warn;
use dotenv::dotenv;

fn main() {
    // Load .env file
    dotenv().ok(); // Need to be loaded before logger init for setting env vars up.

    // Init logger
    env_logger::init();

    // TODO: Check source directory

    // Check and create target directory
    debug!("Checking and creating folder for target mapping.");
    let target_directory = match std::env::var("TARGET_DIR") {
        Ok(target_dir) => {
            info!("TARGET_DIR={}", target_dir);
            target_dir
        },
        Err(_) => {
            error!("Failed to load TARGET_DIR env var. Please check .env file.");
            panic!()
        },
    };
    series::create_folder(&target_directory);
    info!("Finish creating folder {} for target mapping.", target_directory);

    // Iterate through source directory to resolve each anime series
    
}
