/*
* @Author: BlahGeek
* @Date:   2017-04-19
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-28
*/

use std;
use std::error::Error;
use mcore::item::Item;

#[derive(Debug, Clone)]
pub enum ActionArg {
    None,
    Text(String),
    Path(std::path::PathBuf),
}

impl ActionArg {
    pub fn is_none(&self) -> bool {
        match self {
            &ActionArg::None => { true },
            _ => { false }
        }
    }
}

pub type ActionResult = Result<Vec<Item>, Box<Error + Send + Sync>>;

/// The general action type
pub trait Action {

    /// Get an item representing itself
    fn get_item(&self) -> Item;

    /// Whether this action runs without input
    fn accept_nothing(&self) -> bool { false }
    /// Whether this action accepts text input
    fn accept_text(&self) -> bool { false }
    /// Whether this action accepts path input
    fn accept_path(&self) -> bool { false }

    /// Whether this action accepts some arg
    fn accept_arg(&self, arg: &ActionArg) -> bool {
        match arg {
            &ActionArg::None => self.accept_nothing(),
            &ActionArg::Text(_) => self.accept_text(),
            &ActionArg::Path(_) => self.accept_path(),
        }
    }

    /// Whether this action is supposed to return items
    fn should_return_items(&self) -> bool { true }

    /// Auto-complete (suggest) input test
    fn complete_text(&self, &str) -> Result<Vec<String>, Box<Error>> {
        Ok(Vec::new())
    }

    /// Run the action without input
    fn run(&self) -> ActionResult { unimplemented!() }

    /// Run the action with text input
    fn run_text(&self, &str) -> ActionResult { unimplemented!() }

    /// Run the action with path input
    fn run_path(&self, &std::path::Path) -> ActionResult { unimplemented!() }

    /// Run properly function using ActionArg
    fn run_arg(&self, arg: &ActionArg) -> ActionResult {
        match *arg {
            ActionArg::None => self.run(),
            ActionArg::Text(ref text) => self.run_text(&text),
            ActionArg::Path(ref path) => self.run_path(&path),
        }
    }
}

