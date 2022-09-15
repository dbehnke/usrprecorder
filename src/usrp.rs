use serde_derive::Deserialize;
use std::fs::File;
use std::io::{prelude::*};
use toml;

#[derive(Deserialize)]
pub struct Config {
    name: String
}

impl Config {
    pub fn new (name: &str) -> Config {
        Config{name: String::from(name)}
    }

    pub fn load_from_file(filepath: &str) -> Result<Config, Box<dyn std::error::Error>> {
        let mut file = File::open(filepath)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let c: Config = toml::from_str(&contents)?;
        return Ok(c);
        
    }
}
pub fn rx_loop(c: &Config) {
    println!("Loaded Config for {}", c.name)
}
