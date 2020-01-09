use std::fs::File;
use std::io::prelude::*;
use std::convert::TryFrom;


pub struct Config {
    pub thread_count: usize,
    pub test_param: usize,
}

fn get_def_config_toml() -> toml::Value {
    return (r#"
        thread_count = 3
        test_param = 5
    "#).parse().unwrap();
}

fn get_config_param(def_config_toml: &toml::Value, user_config_toml: &toml::Value,
                    param: &str) -> usize {
    let toml_param = user_config_toml.get(param).unwrap_or(&def_config_toml[param]);
    if !toml_param.is_integer() {
        panic!("Error parsing config file: param {} should be a number.", param);
    }
    return usize::try_from(toml_param.as_integer().unwrap()).unwrap();
}

pub fn get_config() -> Config {
    let mut config_params = ["thread_count", "asd"];
    let mut file = File::open("rase.toml").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let user_config_toml: toml::Value = contents.parse().unwrap();
    let def_config_toml = get_def_config_toml();

    let config = Config {
        thread_count: get_config_param(&def_config_toml, &user_config_toml, "thread_count"),
        test_param: get_config_param(&def_config_toml, &user_config_toml, "test_param"),
    };
    return config;
}
