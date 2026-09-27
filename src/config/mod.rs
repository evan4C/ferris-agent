pub mod constants;
pub mod settings;

use config::{Config, Environment, File};
use once_cell::sync::Lazy;
use settings::Settings;

pub static SETTINGS: Lazy<Settings> = Lazy::new(|| {
    dotenvy::dotenv().ok();

    Config::builder()
        .add_source(File::with_name("config").required(false))
        .add_source(Environment::default().separator("__"))
        .build()
        .unwrap()
        .try_deserialize()
        .unwrap()
});
