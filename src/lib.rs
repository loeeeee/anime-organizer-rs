use log::debug;
use log::error;
use log::info;


pub fn load_env_var(var_name: &str) -> Result<String, std::env::VarError> {
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
