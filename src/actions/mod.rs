/*
* @Author: BlahGeek
* @Date:   2017-04-18
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-17
*/

mod capital;
mod linux_desktop_entry;
mod search_engine;

use toml;

use std::fmt;
use std::rc::Rc;
use std::sync::{Mutex, Arc};
use std::error::Error;
use std::path::PathBuf;

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
    for desktop_entry in linux_desktop_entry::LinuxDesktopEntry::get_all(config["linux_desktop_entry"].clone()) {
        ret.push(Rc::new(Box::new(desktop_entry)));
    }
    for se in search_engine::SearchEngine::get_all(config["search_engine"].clone()) {
        ret.push(Rc::new(Box::new(se)));
    }
    ret
}
