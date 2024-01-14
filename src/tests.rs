#[cfg(test)]
mod tests {
    use core::panic;
    use std::{fs, io::Read};

    use anime_organizer_rs::load_env_var;
    use log::{warn, error, info};
    use crate::series::{Series, FilterWords};
    
    
    #[test]
    fn series_name_extraction() {
        use serde::{Deserialize, Serialize};

        // Load env
        dotenvy::from_filename("test.env").unwrap();
        
        // Init logger
        env_logger::init();
        
        #[derive(Serialize, Deserialize)]
        struct SeriesName {
            folder_name: String,
            series_name: String,
        }
        
        // Filter words
        let filter_words = FilterWords::load();

        let tests_series_names_path = load_env_var("TESTS_SERIES_NAMES").unwrap();
        info!("Series Name test sheet: {}", &tests_series_names_path);

        let tests_series_names = match fs::read_to_string(tests_series_names_path) {
            Ok(tests) => {
                info!("Successfully read the test sheet.");
                tests
            },
            Err(e) => {
                error!("Failed to load test sheet, due to {}.", &e);
                panic!();
            }
        };

        let test_sheet: Vec<SeriesName> = serde_json::from_str(&tests_series_names).expect("JSON was not well-formatted");
        // info!("{}", &test_sheet)

        for i in test_sheet.iter() {
            info!("{}: {}", &i.folder_name, &i.series_name);
            assert_eq!(crate::series::extract_series_name(&i.folder_name, &filter_words).unwrap(), i.series_name.to_string());
        };

    }
}