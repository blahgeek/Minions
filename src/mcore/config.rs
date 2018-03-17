// use std::slice::SliceConcatExt;
extern crate serde;

use toml;
use self::serde::de::Deserialize;

use std;
use std::fmt;
use std::io::prelude::*;
use std::error::Error;

#[derive(Debug)]
pub struct ConfigGetError {
    path: Vec<String>,
}

impl Error for ConfigGetError {
    fn description(&self) -> &str {
        "Config not found"
    }
}

impl fmt::Display for ConfigGetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Config not found: {}", self.path.join(":"))
    }
}

struct ConfigValue<'a> {
    path: Vec<String>,
    value: Option<&'a toml::Value>,
}

impl<'a> ConfigValue<'a> {
    fn new(root: Option<&'a toml::Value>) -> ConfigValue<'a> {
        ConfigValue {
            path: Vec::new(),
            value: root,
        }
    }

    fn into_result<'de, T>(self) -> Result<T, ConfigGetError>
            where T: Deserialize<'de> {
        let err = ConfigGetError{path: self.path};
        if let Some(v) = self.value {
            if let Ok(v) = v.clone().try_into::<T>() {
                Ok(v)
            } else {
                Err(err)
            }
        } else {
            Err(err)
        }
    }

    fn into_next(&mut self, s: &str) {
        self.path.push(s.into());
        if let Some(v) = self.value {
            if let Some(m) = v.as_table() {
                self.value = m.get(s);
            }
        }
    }
}

#[derive(Clone)]
pub struct Config {
    default: toml::Value,
    user: Option<toml::Value>,
}

impl Config {
    pub fn new(p: &std::path::Path) -> Config {
        info!("Reading config from {:?}", p);
        let mut content = String::new();
        let fin = std::fs::File::open(p);
        if let Ok(mut fin) = fin {
            let _ = fin.read_to_string(&mut content);
        }
        let userconfig = content.parse::<toml::Value>().ok();
        let defaultconfig = 
            include_str!("../../config/default.toml")
            .parse::<toml::Value>().unwrap();
        Config {
            default: defaultconfig,
            user: userconfig,
        }
    }

    pub fn get<'de, T>(&self, path: &[&str]) -> Result<T, ConfigGetError>
            where T: Deserialize<'de> {
        let mut userval = ConfigValue::new(self.user.as_ref());
        let mut defaultval = ConfigValue::new(Some(&self.default));
        for p in path {
            userval.into_next(p);
            defaultval.into_next(p);
        }
        userval.into_result::<T>().or(defaultval.into_result::<T>())
    }

    /// Same as get::<PathBuf>, but handle paths starting with `~/`
    pub fn get_filename<'de>(&self, path: &[&str]) -> Result<std::path::PathBuf, ConfigGetError> {
        let strval = self.get::<String>(path)?;
        let mut p = std::path::Path::new(&strval).to_path_buf();
        if strval.starts_with("~/") {
            if let Some(homedir) = std::env::home_dir() {
                p = homedir;
                p.push(std::path::Path::new(&strval[2..]));
            }
        }
        Ok(p)
    }

    pub fn partial(&self, path: &[&str]) -> Result<Self, ConfigGetError> {
        let mut userval = ConfigValue::new(self.user.as_ref());
        let mut defaultval = ConfigValue::new(Some(&self.default));
        for p in path {
            userval.into_next(p);
            defaultval.into_next(p);
        }
        Ok(Config {
            default: defaultval.into_result::<toml::Value>()?,
            user: userval.into_result::<toml::Value>().ok(),
        })
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_config_get() {
        let dummyconfig = Config::new(std::path::Path::new(""));

        let v = dummyconfig.get::<i32>(&["core", "filter_timeout"]).unwrap();
        assert_eq!(v, 800);

        let v = dummyconfig.get::<i32>(&["core", "filter_timeoutx"]);
        assert!(v.is_err());

        let v = dummyconfig.get::<String>(&["core", "history_file"]).unwrap();
        assert_eq!(v, "~/.minions/history.dat");

        let v = dummyconfig.get_filename(&["core", "history_file"]).unwrap();
        assert!(v.to_str().unwrap().ends_with("/.minions/history.dat"));
    }
}
