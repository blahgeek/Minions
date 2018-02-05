/*
* @Author: BlahGeek
* @Date:   2017-04-19
* @Last Modified by:   BlahGeek
* @Last Modified time: 2018-02-05
*/

use std;
use std::error::Error;
use mcore::item::Item;

pub type ActionResult = Result<Vec<Item>, Box<Error + Send + Sync>>;

/// The general action type
pub trait Action {

    /// Whether this action runs without input
    fn runnable_bare(&self) -> bool { false }

    /// Whether this action runs with argument
    fn runnable_arg(&self) -> bool { false }

    /// Whether this action runs with argument in realtime
    fn runnable_arg_realtime(&self) -> bool { false }

    /// Whether this action is supposed to return items
    fn should_return_items(&self) -> bool { true }

    /// Run realtime (auto-complete)
    fn run_arg_realtime(&self, &str) -> ActionResult { unimplemented!() }

    /// Run the action without input
    fn run_bare(&self) -> ActionResult { unimplemented!() }

    /// Run the action with text input
    fn run_arg(&self, &str) -> ActionResult { unimplemented!() }

}

