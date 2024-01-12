#[cfg(test)]
mod tests {
    use core::panic;
    use std::{fs, io::Read};

    use anime_organizer_rs::load_env_var;
    use log::{warn, error, info};
    use crate::series::{Series, FilterWords};

    struct SeriesName {
        folder_name: String,
        series_name: String,
    }

    #[test]
    fn series_name_extraction() {
        // Load env
        dotenvy::from_filename("test.env").unwrap();
        
        // Init logger
        env_logger::init();

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

        let test_sheet: serde_json::Value = serde_json::from_str(&tests_series_names).expect("JSON was not well-formatted");
        // info!("{}", &test_sheet)

        for (key, value) in test_sheet.as_object().unwrap() {
            // info!("{}: {}", &key, &value);
            assert_eq!(crate::series::extract_series_name(&key, &filter_words).unwrap(), value.to_string());
        };

    }
}