/*
* @Author: BlahGeek
* @Date:   2017-04-18
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-04-20
*/

use std::error::Error;
use mcore::action::Action;
use mcore::item::Item;

pub struct Capital {}

impl Action for Capital {
    fn name(&self) -> String { "Capital".into() }
    fn accept_text(&self) -> bool { true }
    fn run_text(&self, text: &str) -> Result<Vec<Item>, Box<Error>> {
        let capital = text.to_uppercase();
        let ret = Item::new_text_item(&capital);
        Ok(vec![ret])
    }
}
