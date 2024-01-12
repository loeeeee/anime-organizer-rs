#[cfg(test)]
mod tests {
    use std::fs;

    use anime_organizer_rs::load_env_var;
    use log::{warn, error};
    use crate::series::Series;

    #[test]
    fn series_name_extraction() {
        dotenvy::from_filename("test.env").unwrap();

        let source_dir = load_env_var("SOURCE_DIR").unwrap();
        println!("{}", &source_dir);

        let paths = match fs::read_dir(&source_dir) {
            Ok(paths) => paths,
            Err(e) => {
                error!("Failed open {}, due to {}", &source_dir, &e);
                panic!();
            }
        };

        let series = Series::new(&source_dir);
    }
}