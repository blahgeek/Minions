/*
* @Author: BlahGeek
* @Date:   2017-04-22
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-05-14
*/

use mcore::item::Item;
use mcore::action::Icon;

use gtk;
use gtk::prelude::*;

pub struct MinionsUI {
    window_builder: gtk::Builder,
    pub window: gtk::Window,
    listbox: gtk::ListBox,
    filterlabel: gtk::Label,
    textentry: gtk::Entry,
}

impl MinionsUI {

    pub fn new() -> MinionsUI {
        let window_builder = gtk::Builder::new_from_string(include_str!("resource/minions.glade"));
        let window = window_builder.get_object::<gtk::Window>("root")
                     .expect("Failed to initialize from glade file");
        let listbox = window_builder.get_object::<gtk::ListBox>("listbox").unwrap();
        let label = window_builder.get_object::<gtk::Label>("filter").unwrap();
        let entry = window_builder.get_object::<gtk::Entry>("entry").unwrap();

        window.show_all();
        window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        MinionsUI {
            window_builder: window_builder,
            window: window,
            listbox: listbox,
            filterlabel: label,
            textentry: entry,
        }
    }

    pub fn set_filter_text(&self, text: &str) {
        self.filterlabel.set_text(text);
    }

    pub fn set_reference_item(&self, item: &Option<Item>) {
        let reference = self.window_builder.get_object::<gtk::Box>("reference").unwrap();
        let reference_name = self.window_builder.get_object::<gtk::Label>("reference_name").unwrap();
        match item {
            &None => reference.hide(),
            &Some(ref reference_item) => {
                reference.show();
                reference_name.set_text(&reference_item.title);
            }
        }
    }

    pub fn set_items(&self, items: &Vec<&Item>) {
        for item_ui in self.listbox.get_children().iter() {
            self.listbox.remove(item_ui);
        }
        for item in items.iter() {
            let builder = gtk::Builder::new_from_string(include_str!("resource/item_template.glade"));
            let item_ui = builder.get_object::<gtk::Box>("item_template")
                          .expect("Failed to get item template from glade file");

            let titlebox = builder.get_object::<gtk::Box>("titlebox").unwrap();
            let title = builder.get_object::<gtk::Label>("title").unwrap();
            let subtitle = builder.get_object::<gtk::Label>("subtitle").unwrap();
            let badge = builder.get_object::<gtk::Label>("badge").unwrap();
            let arrow = builder.get_object::<gtk::Image>("arrow").unwrap();
            let icon = builder.get_object::<gtk::Image>("icon").unwrap();

            title.set_text(&item.title);

            if let Some(ref ico) = item.icon {
                match ico {
                    &Icon::Name(ref ico_name) => icon.set_from_icon_name(&ico_name, -1),
                    &Icon::File(ref path) => icon.set_from_file(&path),
                }
            }

            match item.subtitle {
                Some(ref text) => subtitle.set_text(&text),
                None => titlebox.remove(&subtitle),
            }
            match item.badge {
                Some(ref text) => badge.set_text(&text),
                None => item_ui.remove(&badge),
            }

            if match item.action {
                Some(ref action) => !action.should_return_items(),
                None => true,
            } {
                item_ui.remove(&arrow);
            }
            self.listbox.add(&item_ui);
        }
    }
}
