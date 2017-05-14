/*
* @Author: BlahGeek
* @Date:   2017-04-18
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-05-14
*/

mod capital;
mod linux_desktop_entry;

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
    vec![
        Box::new(capital::Capital{}),
    ]
}
