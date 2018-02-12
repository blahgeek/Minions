/*
* @Author: BlahGeek
* @Date:   2017-04-22
* @Last Modified by:   BlahGeek
* @Last Modified time: 2018-02-12
*/

extern crate gdk_pixbuf;
extern crate lru_cache;

use std::cmp;
use std::cell::RefCell;
use std::path::PathBuf;
use std::error::Error;

use mcore::item::{Item, Icon};
use mcore::context::Context;

use frontend::gtk;
use frontend::gtk::prelude::*;
use self::lru_cache::LruCache;

pub struct MinionsUI {
    pub window: gtk::Window,
    pub textentry: gtk::Entry,

    listbox: gtk::ListBox,
    filter_label: gtk::Label,
    icon: gtk::Image,
    icon_text: gtk::Label,
    spinner: gtk::Spinner,
    reference_label: gtk::Label,
    action_box: gtk::Box,
    action_label: gtk::Label,

    gtkbuf_cache: RefCell<LruCache<PathBuf, Option<gdk_pixbuf::Pixbuf>>>,
}

const LISTBOX_NUM: i32 = 5;
const ICON_SIZE: i32 = 45;
const ICON_FONT_SIZE: i32 = 28;
const GTKBUF_CACHE_SIZE: usize = 128;

impl MinionsUI {

    pub fn new() -> MinionsUI {
        let builder = gtk::Builder::new_from_string(include_str!("resource/minions.glade"));
        let window = builder.get_object::<gtk::Window>("root")
                     .expect("Failed to initialize from glade file");
        let spinner = builder.get_object::<gtk::Spinner>("spinner").unwrap();

        window.show_all();
        spinner.hide();

        let style_provider = gtk::CssProvider::new();
        style_provider.load_from_data(include_bytes!("./resource/style.css")).unwrap();
        gtk::StyleContext::add_provider_for_screen(&window.get_screen().unwrap(), &style_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

        MinionsUI {
            window: window,
            spinner: spinner,
            listbox: builder.get_object::<gtk::ListBox>("listbox").unwrap(),
            filter_label: builder.get_object::<gtk::Label>("filter").unwrap(),
            textentry: builder.get_object::<gtk::Entry>("entry").unwrap(),
            icon: builder.get_object::<gtk::Image>("icon").unwrap(),
            icon_text: builder.get_object::<gtk::Label>("icon_text").unwrap(),
            reference_label: builder.get_object::<gtk::Label>("reference").unwrap(),
            action_box: builder.get_object::<gtk::Box>("action_box").unwrap(),
            action_label: builder.get_object::<gtk::Label>("action_name").unwrap(),
            gtkbuf_cache: RefCell::new(LruCache::new(GTKBUF_CACHE_SIZE)),
        }
    }

    pub fn set_spinning(&self, v: bool) {
        if v { self.spinner.show(); }
        else { self.spinner.hide(); }
    }

    fn set_image_icon(&self, w_image: &gtk::Image, w_label: &gtk::Label, icon: &Icon) {
        match icon {
            &Icon::GtkName(ref ico_name) => {
                w_image.set_from_icon_name(Some(ico_name.as_str()), gtk::IconSize::Button.into());
                w_image.set_pixel_size(ICON_SIZE);
                w_image.show();
                w_label.hide();
            },
            &Icon::File(ref path) => {
                let mut gtkbuf_cache = self.gtkbuf_cache.borrow_mut();
                let buf = if let Some(pixbuf) = gtkbuf_cache.get_mut(path) {
                    pixbuf.clone()
                } else {
                    gdk_pixbuf::Pixbuf::new_from_file_at_size(&path.to_string_lossy(), ICON_SIZE, ICON_SIZE).ok()
                };
                w_image.set_from_pixbuf(buf.as_ref());
                gtkbuf_cache.insert(path.clone(), buf);
                w_image.show();
                w_label.hide();
            },
            &Icon::Character{ref ch, ref font} => {
                w_label.set_markup(&format!("<span font_desc=\"{} {}\">{}</span>", font, ICON_FONT_SIZE, ch));
                w_image.hide();
                w_label.show();
            },
        }
    }


    pub fn set_entry(&self, item: Option<&Item>) {
        if let Some(item) = item {
            self.textentry.set_text(&item.title);
            if let Some(ref ico) = item.icon {
                self.set_image_icon(&self.icon, &self.icon_text, ico);
            } else {
                self.set_image_icon(&self.icon, &self.icon_text, &Icon::Character{ch: '', font: "FontAwesome".into()} );
            }
        } else {
            self.textentry.set_buffer(&gtk::EntryBuffer::new(None));
            self.set_image_icon(&self.icon, &self.icon_text, &Icon::Character{ch: '', font: "FontAwesome".into()} );
        }
        self.textentry.set_can_focus(false);
        self.textentry.set_editable(false);
        self.window.set_focus::<gtk::Entry, Option<&gtk::Entry>>(None);
    }

    pub fn set_entry_editable(&self) {
        if !self.textentry.get_editable() {
            self.textentry.set_text("");
            self.textentry.set_editable(true);
            self.textentry.set_can_focus(true);
            self.textentry.grab_focus();
        }
    }

    pub fn get_entry_text(&self) -> String {
        self.textentry.get_text().unwrap_or(String::new())
    }

    pub fn set_filter_text(&self, text: &str) {
        self.filter_label.set_text(text);
    }

    pub fn set_error(&self, error: &Box<Error>) {
        self.reference_label.set_text(&format!("{}: {}", error.description(), error));
        self.reference_label.show();
        self.set_image_icon(&self.icon, &self.icon_text, &Icon::GtkName("dialog-warning".into()));
    }

    pub fn set_reference(&self, reference: Option<&String>) {
        if let Some(text) = reference {
            self.reference_label.set_text(&text);
            self.set_action_name(Some("Quicksend"));
            self.reference_label.show();
        } else {
            self.reference_label.hide();
        }
    }

    fn set_action_name(&self, name: Option<&str>) {
        if let Some(name) = name {
            self.action_label.set_text(name);
            self.action_box.show();
        } else {
            self.action_box.hide();
        }
    }

    pub fn set_action(&self, item: Option<&Item>) {
        if let Some(item) = item {
            if let Some(ref ico) = item.icon {
                self.set_image_icon(&self.icon, &self.icon_text, ico);
            }
            self.set_action_name(Some(&item.title));
        } else {
            self.set_action_name(None)
        }
    }

    fn build_item(&self, item: &Item, ctx: &Context) -> gtk::Box {
        let builder = gtk::Builder::new_from_string(include_str!("resource/item_template.glade"));
        let item_ui = builder.get_object::<gtk::Box>("item_template")
                      .expect("Failed to get item template from glade file");

        let titlebox = builder.get_object::<gtk::Box>("titlebox").unwrap();
        let title = builder.get_object::<gtk::Label>("title").unwrap();
        let subtitle = builder.get_object::<gtk::Label>("subtitle").unwrap();
        let badge = builder.get_object::<gtk::Label>("badge").unwrap();
        let selectable = builder.get_object::<gtk::Label>("selectable").unwrap();
        let icon = builder.get_object::<gtk::Image>("icon").unwrap();
        let icon_text = builder.get_object::<gtk::Label>("icon_text").unwrap();

        title.set_text(&item.title);

        if let Some(ref ico) = item.icon {
            self.set_image_icon(&icon, &icon_text, ico);
        } else {
            self.set_image_icon(&icon, &icon_text, &Icon::Character{ch: '', font: "FontAwesome".into()});
        }

        match item.subtitle {
            Some(ref text) => if text.len() > 0 {
                subtitle.set_text(&text)
            } else {
                titlebox.remove(&subtitle)
            },
            None => titlebox.remove(&subtitle),
        }
        match item.badge {
            Some(ref text) => badge.set_text(&text),
            None => item_ui.remove(&badge),
        }

        if ctx.selectable(&item) {
            selectable.set_text("");
        } else if ctx.selectable_with_text(&item) {
            selectable.set_text("");
        } else {
            selectable.set_text(" ");
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
            let item_ui = self.build_item(items[i as usize], ctx);
            self.listbox.add(&item_ui);
        }

        if highlight < 0 {
            self.listbox.select_row(None);
        } else {
            self.listbox.select_row(self.listbox.get_row_at_index(highlight - display_start).as_ref());
        }
    }

}
