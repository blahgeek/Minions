/*
* @Author: BlahGeek
* @Date:   2017-04-22
* @Last Modified by:   BlahGeek
* @Last Modified time: 2020-01-17
*/

extern crate gdk_pixbuf;
extern crate lru_cache;
extern crate serde_json;

use std::cmp;
use std::cell::RefCell;
use std::path::PathBuf;

use crate::mcore::item::{Item, Icon};
use crate::mcore::context::Context;
use crate::mcore::errors::Error;

use crate::frontend::gtk;
use crate::frontend::gtk::prelude::*;
use self::gdk_pixbuf::prelude::*;
use self::lru_cache::LruCache;

use error_chain::ChainedError;


pub struct ItemUI {
    title: gtk::Label,
    subtitle: gtk::Label,
    badge: gtk::Label,
    selectable: gtk::Label,
    icon: gtk::Image,
    icon_text: gtk::Label,
}

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
    items: Vec<ItemUI>,
}

lazy_static! {
    pub static ref FA_FONTS : serde_json::Value =
        serde_json::from_str(include_str!("./resource/fontawesome/icons.json")).unwrap();
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
        let listbox = builder.get_object::<gtk::ListBox>("listbox").unwrap();

        window.show_all();
        spinner.hide();

        let style_provider = gtk::CssProvider::new();
        style_provider.load_from_data(include_bytes!("./resource/style.css")).unwrap();
        gtk::StyleContext::add_provider_for_screen(&window.get_screen().unwrap(), &style_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

        let mut items = Vec::new();
        for _ in 0..LISTBOX_NUM {
            let builder = gtk::Builder::new_from_string(include_str!("resource/item_template.glade"));
            let item_ui_top = builder.get_object::<gtk::Box>("item_template").unwrap();
            items.push(ItemUI{
                title: builder.get_object::<gtk::Label>("title").unwrap(),
                subtitle: builder.get_object::<gtk::Label>("subtitle").unwrap(),
                badge: builder.get_object::<gtk::Label>("badge").unwrap(),
                selectable: builder.get_object::<gtk::Label>("selectable").unwrap(),
                icon: builder.get_object::<gtk::Image>("icon").unwrap(),
                icon_text: builder.get_object::<gtk::Label>("icon_text").unwrap(),
            });
            listbox.add(&item_ui_top);
        }

        MinionsUI {
            window: window,
            spinner: spinner,
            listbox: listbox,
            filter_label: builder.get_object::<gtk::Label>("filter").unwrap(),
            textentry: builder.get_object::<gtk::Entry>("entry").unwrap(),
            icon: builder.get_object::<gtk::Image>("icon").unwrap(),
            icon_text: builder.get_object::<gtk::Label>("icon_text").unwrap(),
            reference_label: builder.get_object::<gtk::Label>("reference").unwrap(),
            action_box: builder.get_object::<gtk::Box>("action_box").unwrap(),
            action_label: builder.get_object::<gtk::Label>("action_name").unwrap(),
            gtkbuf_cache: RefCell::new(LruCache::new(GTKBUF_CACHE_SIZE)),
            items: items,
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
                    gdk_pixbuf::Pixbuf::new_from_file_at_size(path, ICON_SIZE, ICON_SIZE).ok()
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
            &Icon::FontAwesome(ref name) => {
                let mut pixbuf = gdk_pixbuf::Pixbuf::new(gdk_pixbuf::Colorspace::Rgb, true, 8, ICON_SIZE, ICON_SIZE);
                let svg = FA_FONTS.get(&name)
                    .and_then(|x| x.get("svg"))
                    .and_then(|x| x.get("brands").or_else(|| x.get("solid")))
                    .and_then(|x| x.get("raw"))
                    .and_then(|x| x.as_str());
                if let Some(svg) = svg {
                    let color = self.window.get_style_context().get_color(gtk::StateFlags::NORMAL);
                    let red = (color.red * 256.0) as i32;
                    let green = (color.green * 256.0) as i32;
                    let blue = (color.blue * 256.0) as i32;
                    let svg = svg.replace("<path",
                                          &format!("<path stroke=\"rgb({},{},{})\" fill=\"rgb({},{},{})\"",
                                                   red, green, blue,
                                                   red, green, blue));
                    let loader = gdk_pixbuf::PixbufLoader::new();
                    let _ = loader.write(svg.as_bytes());
                    let _ = loader.close();
                    pixbuf = loader.get_pixbuf()
                        .and_then(|x| {
                            if x.get_width() >= x.get_height() {
                                x.scale_simple(ICON_SIZE,
                                    ((ICON_SIZE as f32) / (x.get_width() as f32) * (x.get_height() as f32)) as i32,
                                    gdk_pixbuf::InterpType::Bilinear)
                            } else {
                                x.scale_simple(((ICON_SIZE as f32) / (x.get_height() as f32) * (x.get_width() as f32)) as i32,
                                ICON_SIZE,
                                gdk_pixbuf::InterpType::Bilinear)
                            }
                        });
                }
                w_image.set_from_pixbuf(pixbuf.as_ref());
                w_image.show();
                w_label.hide();
            },
        }
    }


    pub fn set_entry(&self, item: Option<&Item>) {
        if let Some(item) = item {
            self.textentry.set_text(&item.title);
            if let Some(ref ico) = item.icon {
                self.set_image_icon(&self.icon, &self.icon_text, ico);
            } else {
                self.set_image_icon(&self.icon, &self.icon_text, &Icon::FontAwesome("home".into()) );
            }
        } else {
            self.textentry.set_buffer(&gtk::EntryBuffer::new(None));
            self.set_image_icon(&self.icon, &self.icon_text, &Icon::FontAwesome("home".into()) );
        }
        self.textentry.set_can_focus(false);
        self.textentry.set_editable(false);
        self.window.set_focus::<gtk::Entry>(None);
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
        self.textentry.get_text().and_then(|x| Some(x.as_str().to_owned())).unwrap_or(String::new())
    }

    pub fn set_filter_text(&self, text: &str) {
        self.filter_label.set_text(text);
    }

    pub fn set_error(&self, error: &Error) {
        self.reference_label.set_text(&error.display_chain().to_string());
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

    fn update_item(&self, idx: usize, item: &Item, ctx: &Context) {
        let item_ui = &self.items[idx];

        item_ui.title.set_text(&item.title);
        if let Some(ref ico) = item.icon {
            self.set_image_icon(&item_ui.icon, &item_ui.icon_text, ico);
        } else {
            self.set_image_icon(&item_ui.icon, &item_ui.icon_text, &Icon::FontAwesome("info-circle".into()) );
        }

        match item.subtitle {
            Some(ref text) => if text.len() > 0 {
                item_ui.subtitle.show();
                item_ui.subtitle.set_text(&text);
            } else {
                item_ui.subtitle.hide();
            },
            None => item_ui.subtitle.hide(),
        }
        match item.badge {
            Some(ref text) => { item_ui.badge.set_text(&text); item_ui.badge.show(); },
            None => item_ui.badge.hide(),
        }

        if ctx.selectable(&item) {
            item_ui.selectable.set_text(">");
        } else if ctx.selectable_with_text(&item) {
            item_ui.selectable.set_text("A");
        } else {
            item_ui.selectable.set_text(" ");
        }
    }

    pub fn set_items(&self, items: Vec<&Item>, highlight: i32, ctx: &Context) {

        let mut display_start =
            if highlight < (LISTBOX_NUM / 2) { 0 }
            else { highlight - (LISTBOX_NUM / 2) };
        let display_end = cmp::min(display_start + LISTBOX_NUM, items.len() as i32);

        if display_end - display_start < LISTBOX_NUM {
            display_start = cmp::max(0, display_end - LISTBOX_NUM);
        }

        trace!("display: {}:{}", display_start, display_end);
        for i in display_start .. display_end {
            self.update_item((i - display_start) as usize, items[i as usize], ctx);
            self.listbox.get_row_at_index(i - display_start).unwrap().show();
        }
        for i in (display_end - display_start) .. LISTBOX_NUM {
            self.listbox.get_row_at_index(i).unwrap().hide();
        }

        if highlight < 0 {
            self.listbox.select_row::<gtk::ListBoxRow>(None);
        } else {
            self.listbox.select_row(self.listbox.get_row_at_index(highlight - display_start).as_ref());
        }
    }

}
