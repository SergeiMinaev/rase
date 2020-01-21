use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::convert::TryFrom;
use log::{error};

#[derive(Clone)]
pub struct Config {
    pub address: String,
    pub port: String,
    pub address_full: String,
    pub thread_count: usize,
    pub test_param: usize,
    pub static_dir: String,
    pub static_url: String,
}

fn get_def_config_toml() -> toml::Value {
    return (r#"
        address = '127.0.0.1'
        port = '8000'
        address_full = '127.0.0.1:8000'
        thread_count = 3
        test_param = 5
        static_dir = ''
        static_url = '/static/'
    "#).parse().unwrap();
}

fn get_config_param_num(def_config_toml: &toml::Value,
                        user_config_toml: &toml::Value,
                        param: &String,
                        is_required: bool) -> usize {
    if is_required && !user_config_toml.get(param).is_some() {
        panic!("Error parsing config file: param {} is required.", param);
    }
    let p = user_config_toml.get(param).unwrap_or(&def_config_toml[param]);
    match p.as_integer() {
        None => {
            error!("Error while parsing config file: \
                    param {} should be a number.", param);
            std::process::exit(0);
        },
        Some(r) => {
            match usize::try_from(r) {
                Err(err) => panic!(err),
                Ok(r) => return r,
            }
        }
    };
}

fn get_config_param_str(def_config_toml: &toml::Value,
                        user_config_toml: &toml::Value,
                        param: &String,
                        is_required: bool) -> String {
    if is_required && !user_config_toml.get(param).is_some() {
        panic!("Error parsing config file: param {} is required.", param);
    }
    let p = user_config_toml.get(param).unwrap_or(&def_config_toml[param]);
    match p.as_str() {
        None => panic!("Error in config file: param {} should be a string.", param),
        Some(r) => return String::from(r),
    };
}

pub fn get_config() ->  Config {
    let path = Path::new("rase.toml");
    let mut file = match File::open(&path) {
        Err(why) => {
            error!("Couldn't open config file {}: {}\n\
                Config file is required. You can find an example config \
                in Rase dir.", path.display(), why);
            std::process::exit(0);
        },
        Ok(file) => file,
    };
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Err(why) => {
            error!("Couldn't read {}: {}", path.display(), why);
            std::process::exit(0);
        },
        Ok(c) => c,
    };

    let user_config_toml: toml::Value = match contents.parse() {
        Err(why) => {
            error!("Couldn't parse config file {}: {}", path.display(), why);
            std::process::exit(0);
        },
        Ok(c) => c,
    };
    let def_config_toml = get_def_config_toml();
    
    let address = String::from(&get_config_param_str(&def_config_toml,
                                    &user_config_toml,
                                    &"address".to_string(), false));
    let port = String::from(get_config_param_str(&def_config_toml,
                                    &user_config_toml, 
                                    &"port".to_string(), false));
    let address_full = [&address, ":", &port].join("").to_string();
    let mut config = Config {
        address: address,
        port: port,
        address_full: address_full,
        thread_count: get_config_param_num(&def_config_toml,
                                    &user_config_toml, 
                                    &"thread_count".to_string(), false),
        test_param: get_config_param_num(&def_config_toml,
                                    &user_config_toml,
                                    &"test_param".to_string(), false),
        static_dir: String::from(&get_config_param_str(&def_config_toml,
                                    &user_config_toml,
                                    &"static_dir".to_string(), true)),
        static_url: String::from(&get_config_param_str(&def_config_toml,
                                    &user_config_toml,
                                    &"static_url".to_string(), true)),
    };
    if !config.static_url.ends_with("/") {
        config.static_url.push_str("/");
    }
    return config;
}
