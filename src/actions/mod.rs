/*
* @Author: BlahGeek
* @Date:   2017-04-18
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-04-20
*/

mod capital;

use mcore::action::Action;

pub fn get_actions() -> Vec<Box<Action>> {
    vec![
        Box::new(capital::Capital{}),
    ]
}
