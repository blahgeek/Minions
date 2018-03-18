/*
* @Author: BlahGeek
* @Date:   2017-04-19
* @Last Modified by:   BlahGeek
* @Last Modified time: 2018-03-18
*/

use std::sync::Arc;
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

    /// Scope of entered argument history, only valid if runnable_arg
    fn suggest_arg_scope(&self) -> Option<&str> { None }

    /// Run realtime (auto-complete)
    fn run_arg_realtime(&self, &str) -> ActionResult { unimplemented!() }

    /// Run the action without input
    fn run_bare(&self) -> ActionResult { unimplemented!() }

    /// Run the action with text input
    fn run_arg(&self, &str) -> ActionResult { unimplemented!() }

}

/// An actiton with arg
pub struct PartialAction {
    action: Arc<Box<Action + Sync + Send>>,
    arg: String,
}

impl PartialAction {
    pub fn new(action: Arc<Box<Action + Sync + Send>>, arg: String) -> Self {
        PartialAction { action, arg, }
    }
}

impl Action for PartialAction {

    fn runnable_bare(&self) -> bool { true }

    fn run_bare(&self) -> ActionResult { self.action.run_arg(&self.arg) }

}
