/*
* @Author: BlahGeek
* @Date:   2017-04-21
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-04-22
*/

extern crate gtk;
extern crate minions;

use std::fmt;
use std::rc::Rc;

use gtk::prelude::*;

use minions::mcore::context::Context;
use minions::mcore::item::Item;
use minions::frontend::ui::MinionsUI;


fn main() {
    gtk::init().expect("Failed to initialize GTK");

    let minions_ui = MinionsUI::new();
    let context = Context::new();

    minions_ui.set_items(&context.list_items.iter().collect());
    minions_ui.set_reference_item(&context.reference_item);

    minions_ui.window.connect_key_press_event(|_, event| {
        println!("Key pressed: {:?}", event);
        gtk::Inhibit(true)
    });

    gtk::main();
}

