/*
* @Author: BlahGeek
* @Date:   2017-04-18
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-17
*/

mod capital;
mod linux_desktop_entry;
mod search_engine;

use std::fmt;
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

pub fn get_actions() -> Vec<Box<Action>> {
    let mut ret : Vec<Box<Action>> = vec![
        Box::new(capital::Capital{}),
    ];
    for desktop_entry in linux_desktop_entry::LinuxDesktopEntry::get_all().into_iter() {
        ret.push(Box::new(desktop_entry));
    }
    for se in search_engine::SearchEngine::get_all() {
        ret.push(Box::new(se));
    }
    ret
}
