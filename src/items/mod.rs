/*
* @Author: BlahGeek
* @Date:   2017-06-14
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-15
*/

use actions;

use mcore::item::Item;

pub fn get_initial_items() -> Vec<Item> {
    actions::get_actions().into_iter()
        .filter(|action| {
            action.accept_nothing() || action.accept_text()
        })
        .map(|action| Item::new_action_item(action) )
        .collect()
}
