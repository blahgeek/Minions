/*
* @Author: BlahGeek
* @Date:   2017-04-18
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-27
*/

use std::error::Error;
use mcore::action::{Action, ActionResult};
use mcore::item::Item;

pub struct Capital {}

impl Action for Capital {
    fn get_item(&self) -> Item {
        Item::new("Capital")
    }
    fn should_return_items(&self) -> bool { true }
    fn accept_text(&self) -> bool { true }
    fn run_text(&self, text: &str) -> ActionResult {
        let capital = text.to_uppercase();
        let ret = Item::new_text_item(&capital);
        Ok(vec![ret])
    }
}
