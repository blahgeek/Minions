/*
* @Author: BlahGeek
* @Date:   2017-04-21
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-04-21
*/

use actions;
use mcore::action::ActionArg;
use mcore::item::{ItemData, Item};


pub fn quicksend(data: &ItemData) -> Vec<Item> {
    match data {
        &ItemData::Text(ref text) => {
            actions::get_actions().into_iter()
            .filter(|action| action.accept_text())
            .map(|action| {
                let mut item = Item::new_action_item(action);
                item.action_arg = ActionArg::Text(text.clone());
                item
            })
            .collect()
        },
        &ItemData::Path(ref path) => {
            actions::get_actions().into_iter()
            .filter(|action| action.accept_path())
            .map(|action| {
                let mut item = Item::new_action_item(action);
                item.action_arg = ActionArg::Path(path.into());
                item
            })
            .collect()
        },
    }
}
