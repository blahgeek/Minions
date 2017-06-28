/*
* @Author: BlahGeek
* @Date:   2017-04-22
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-28
*/

extern crate gdk_pixbuf;

use std::cmp;

use mcore::item::{Item, Icon};
use mcore::context::Context;

use frontend_gtk::gtk;
use frontend_gtk::gtk::prelude::*;

pub struct MinionsUI {
    window_builder: gtk::Builder,
    pub window: gtk::Window,
    listbox: gtk::ListBox,
    filterlabel: gtk::Label,
    textentry: gtk::Entry,
    icon: gtk::Image,
    spinner: gtk::Spinner,
}

static LISTBOX_NUM: i32 = 5;
static ICON_SIZE: i32 = 45;

fn set_image_icon(image: &gtk::Image, icon: &Icon) {
    match icon {
        &Icon::GtkName(ref ico_name) => {
            image.set_from_icon_name(&ico_name, gtk::IconSize::Button.into());
            image.set_pixel_size(ICON_SIZE);
        },
        &Icon::File(ref path) => {
            image.set_from_pixbuf(gdk_pixbuf::Pixbuf::new_from_file_at_size(&path.to_string_lossy(), ICON_SIZE, ICON_SIZE).ok().as_ref())
        },
    }
}

impl MinionsUI {

    pub fn new() -> MinionsUI {
        let window_builder = gtk::Builder::new_from_string(include_str!("resource/minions.glade"));
        let window = window_builder.get_object::<gtk::Window>("root")
                     .expect("Failed to initialize from glade file");
        let listbox = window_builder.get_object::<gtk::ListBox>("listbox").unwrap();
        let label = window_builder.get_object::<gtk::Label>("filter").unwrap();
        let entry = window_builder.get_object::<gtk::Entry>("entry").unwrap();
        let icon = window_builder.get_object::<gtk::Image>("icon").unwrap();
        let spinner = window_builder.get_object::<gtk::Spinner>("spinner").unwrap();

        window.show_all();
        spinner.hide();
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
            icon: icon,
            spinner: spinner,
        }
    }

    pub fn set_spinning(&self, v: bool) {
        if v { self.spinner.show(); }
        else { self.spinner.hide(); }
    }

    pub fn set_entry(&self, item: Option<&Item>) {
        if let Some(item) = item {
            self.textentry.set_text(&item.title);
            if let Some(ref ico) = item.icon {
                set_image_icon(&self.icon, ico);
            } else {
                self.icon.set_from_icon_name("gtk-home", gtk::IconSize::Button.into());
                self.icon.set_pixel_size(ICON_SIZE);
            }
        } else {
            self.textentry.set_text("Minions");
            self.icon.set_from_icon_name("gtk-home", gtk::IconSize::Button.into());
            self.icon.set_pixel_size(ICON_SIZE);
        }
        self.textentry.set_can_focus(false);
        self.textentry.set_editable(false);
    }

    pub fn set_entry_editable(&self) {
        self.textentry.set_text("");
        self.textentry.set_editable(true);
        self.textentry.set_can_focus(true);
        self.textentry.grab_focus();
    }

    pub fn get_entry_text(&self) -> String {
        self.textentry.get_text().unwrap_or(String::new())
    }

    pub fn set_filter_text(&self, text: &str) {
        self.filterlabel.set_text(text);
    }

    pub fn set_reference_item(&self, item: Option<&Item>) {
        let reference = self.window_builder.get_object::<gtk::Box>("reference").unwrap();
        let reference_name = self.window_builder.get_object::<gtk::Label>("reference_name").unwrap();
        match item {
            None => reference.hide(),
            Some(ref reference_item) => {
                reference.show();
                reference_name.set_text(&reference_item.title);
            }
        }
    }

    fn build_item(item: &Item, ctx: &Context) -> gtk::Box {
        let builder = gtk::Builder::new_from_string(include_str!("resource/item_template.glade"));
        let item_ui = builder.get_object::<gtk::Box>("item_template")
                      .expect("Failed to get item template from glade file");

        let titlebox = builder.get_object::<gtk::Box>("titlebox").unwrap();
        let title = builder.get_object::<gtk::Label>("title").unwrap();
        let subtitle = builder.get_object::<gtk::Label>("subtitle").unwrap();
        let badge = builder.get_object::<gtk::Label>("badge").unwrap();
        let selectable = builder.get_object::<gtk::Image>("selectable").unwrap();
        let arrow = builder.get_object::<gtk::Image>("arrow").unwrap();
        let icon = builder.get_object::<gtk::Image>("icon").unwrap();

        title.set_text(&item.title);

        if let Some(ref ico) = item.icon {
            set_image_icon(&icon, ico);
        } else {
            icon.set_from_icon_name("gtk-missing-image", gtk::IconSize::Button.into());
            icon.set_pixel_size(ICON_SIZE);
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
            arrow.set_from_icon_name("gtk-media-stop", gtk::IconSize::SmallToolbar.into());
        } else {
            arrow.set_from_icon_name("gtk-goto-last", gtk::IconSize::SmallToolbar.into());
        }

        if ctx.selectable(&item) {
            selectable.set_from_icon_name("gtk-apply", gtk::IconSize::SmallToolbar.into());
        } else if ctx.selectable_with_text(&item) {
            selectable.set_from_icon_name("gtk-edit", gtk::IconSize::SmallToolbar.into());
        } else {
            item_ui.remove(&selectable);
        }

        item_ui
    }

    pub fn set_items(&self, items: Vec<&Item>, highlight: i32, ctx: &Context) {
        for item_ui in self.listbox.get_children().iter() {
            self.listbox.remove(item_ui);
        }

        let mut display_start =
            if highlight < (LISTBOX_NUM / 2) { 0 }
            else { highlight - (LISTBOX_NUM / 2) };
        let display_end = cmp::min(display_start + LISTBOX_NUM, items.len() as i32);

        if display_end - display_start < LISTBOX_NUM {
            display_start = cmp::max(0, display_end - LISTBOX_NUM);
        }

        trace!("display: {}:{}", display_start, display_end);
        for i in display_start .. display_end {
            let item_ui = MinionsUI::build_item(items[i as usize], ctx);
            self.listbox.add(&item_ui);
        }

        if highlight < 0 {
            self.listbox.select_row(None);
        } else {
            self.listbox.select_row(self.listbox.get_row_at_index(highlight - display_start).as_ref());
        }
    }

}
