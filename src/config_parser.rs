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
    pub gzip_min_size: usize,
    pub gzip_max_size: usize,
    pub gzip_file_types: std::vec::Vec<String>,
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
        gzip_min_size = 1024
        gzip_max_size = 1048576
        gzip_file_types = ["js", "txt", "html", "css"]
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

fn get_config_param_arr(def_config_toml: &toml::Value,
                        user_config_toml: &toml::Value,
                        param: &String,
                        is_required: bool) -> std::vec::Vec<String> {
    if is_required && !user_config_toml.get(param).is_some() {
        error!("Error parsing config file: param {} is required.", param);
        std::process::exit(0);
    }
    let p = user_config_toml.get(param).unwrap_or(&def_config_toml[param]);
    //return p.as_array().unwrap();
    let arr = match p.as_array() {
        None => panic!("Error in config file: param {} should be a blabla.", param),
        Some(p) => p,
    };
    let mut parsed_arr: std::vec::Vec<String> = Vec::new();
    for item in arr {
        if item.is_str() {
            match item.as_str() {
                None => error!("CANNOT BE EMPTY BLABLA"),
                Some(v) => {
                    if v == "" {
                        error!("Error in config file: param {} should not contain \
                        an empty string.", param);
                        std::process::exit(0);
                    } else {
                        parsed_arr.push(v.to_owned());
                    }
                }
            }
        } else {
            error!("Error in config file: param {} should be an array of strings.",
                   param);
            std::process::exit(0);
        }
    }
    return parsed_arr;
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
        gzip_min_size: get_config_param_num(&def_config_toml,
                                    &user_config_toml, 
                                    &"gzip_min_size".to_string(), false),
        gzip_max_size: get_config_param_num(&def_config_toml,
                                    &user_config_toml, 
                                    &"gzip_max_size".to_string(), false),
        gzip_file_types: get_config_param_arr(&def_config_toml,
                                    &user_config_toml,
                                    &"gzip_file_types".to_string(), false),
    };
    if !config.static_url.ends_with("/") {
        config.static_url.push_str("/");
    }
    return config;
}
