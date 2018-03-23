/*
* @Author: BlahGeek
* @Date:   2017-04-19
* @Last Modified by:   BlahGeek
* @Last Modified time: 2018-03-23
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

    /// Whether the results of runnable_arg_realtime are suggestions
    /// aka, the results' action should be run_arg with item.title
    /// in which case, the results' action should be None and inserted by core context (for lrudb)
    /// only valid if runnable_arg_realtime
    fn runnable_arg_realtime_is_suggestion(&self) -> bool { false }

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

    run_callback: Option<Box<Fn() + Sync + Send + 'static>>,
}

impl PartialAction {
    pub fn new(action: Arc<Box<Action + Sync + Send>>,
               arg: String,
               run_callback: Option<Box<Fn() + Sync + Send + 'static>>) -> Self {
        PartialAction { action, arg, run_callback, }
    }
}

impl Action for PartialAction {

    fn runnable_bare(&self) -> bool { true }

    fn run_bare(&self) -> ActionResult {
        if let Some(ref f) = self.run_callback {
            f();
        }
        self.action.run_arg(&self.arg)
    }

}
