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
            assert_eq!(crate::series::extract_series_name(&i.folder_name, &filter_words).unwrap(), i.series_name.to_string());
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
            assert_eq!(crate::series::extract_series_season_number(&i.folder_name, &filter_words).unwrap().to_string(), i.season_number.to_string());
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
        use crate::series::string_remove_duplicate_spaces;
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
        use crate::series::string_remove_years;
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
        use crate::series::string_find_episode_number;
        for i in test_sheet.iter() {
            info!("{}: {}", &i.file_name, &i.episode_number);
            assert_eq!(string_find_episode_number(&i.file_name).unwrap()[0], i.episode_number);
        }
    }
}