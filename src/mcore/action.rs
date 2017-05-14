/*
* @Author: BlahGeek
* @Date:   2017-04-19
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-05-13
*/

use std;
use std::error::Error;
use mcore::item::Item;

#[derive(Debug)]
pub enum ActionArg {
    None,
    Text(String),
    Path(std::path::PathBuf),
}

/// The general action type
pub trait Action {
    // metadata
    fn name(&self) -> &str;
    // fn icon(&self); // TODO

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
    fn run(&self) -> Result<Vec<Item>, Box<Error>> { unimplemented!() }

    /// Run the action with text input
    fn run_text(&self, &str) -> Result<Vec<Item>, Box<Error>> { unimplemented!() }

    /// Run the action with path input
    fn run_path(&self, &std::path::Path) -> Result<Vec<Item>, Box<Error>> { unimplemented!() }

    /// Run properly function using ActionArg
    fn run_arg(&self, arg: &ActionArg) -> Result<Vec<Item>, Box<Error>> {
        match *arg {
            ActionArg::None => self.run(),
            ActionArg::Text(ref text) => self.run_text(&text),
            ActionArg::Path(ref path) => self.run_path(&path),
        }
    }
}

