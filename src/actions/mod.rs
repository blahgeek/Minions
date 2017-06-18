/*
* @Author: BlahGeek
* @Date:   2017-04-18
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-18
*/

mod capital;
mod linux_desktop_entry;
mod search_engine;
mod file_browser;
mod custom_script;

use toml;

use std::fmt;
use std::rc::Rc;
use std::error::Error;
use std::path::{PathBuf, Path};

#[derive(Clone)]
#[derive(Debug)]
enum ActionError {
    FileFormatError(PathBuf),
    NotSupported,
    Unknown,
}

impl Error for ActionError {
    fn description(&self) -> &str {
        "Error generation action"
    }
}
impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

use mcore::action::Action;

// WTF: I want to cache all actions using lazy_static!
// But I cannot do that since static variables requires Sync and Send
// which is not possible because I used Rc ...

pub fn get_actions(config: toml::Value) -> Vec<Rc<Box<Action>>> {
    let mut ret : Vec<Rc<Box<Action>>> = vec![
        Rc::new(Box::new(capital::Capital{})),
    ];
    if let Some(opts) = config.get("linux_desktop_entry") {
        for desktop_entry in linux_desktop_entry::LinuxDesktopEntry::get_all(opts.clone()) {
            ret.push(Rc::new(Box::new(desktop_entry)));
        }
    }
    if let Some(opts) = config.get("search_engine") {
        for se in search_engine::SearchEngine::get_all(opts.clone()) {
            ret.push(Rc::new(Box::new(se)));
        }
    }
    if let Some(opts) = config.get("file_browser") {
        for x in file_browser::FileBrowserEntry::get_all(opts.clone()) {
            ret.push(Rc::new(Box::new(x)));
        }
    }
    if let Some(opts) = config.get("plugin_directories") {
        if let Some(opts) = opts.as_array() {
            for dir in opts {
                if let Some(dir) = dir.as_str() {
                    for x in custom_script::ScriptAction::get_all(Path::new(dir)) {
                        ret.push(Rc::new(Box::new(x)));
                    }
                }
            }
        }
    }
    ret
}
