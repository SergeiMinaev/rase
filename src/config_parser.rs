use std::fs::File;
use std::io::prelude::*;
use std::convert::TryFrom;


pub struct Config {
    pub thread_count: usize,
}

fn get_def_config() -> Config {
    return Config {
        thread_count: 100,
    };
}

pub fn get_config() -> Config {
    let mut file = File::open("rase.toml").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let config_toml: toml::Value = contents.parse().unwrap();
    let mut config = get_def_config();

    let thread_count = config_toml.get("thread_count");
    if thread_count.is_some() {
        if !thread_count.unwrap().is_integer() {
            panic!("Error parsing config file: thread_count should be a number.");
        }
        config.thread_count = usize::try_from(thread_count.unwrap().as_integer().unwrap()
                                              ).unwrap();
    }

    return config;
}
