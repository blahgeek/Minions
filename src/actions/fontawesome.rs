use frontend::ui::FA_FONTS;

use std::sync::Arc;
use mcore::action::{Action, ActionResult};
use mcore::item::{Item, Icon};
use mcore::config::Config;

struct FontAwesome {}

impl Action for FontAwesome {
    fn runnable_bare(&self) -> bool { true }
    fn run_bare(&self) -> ActionResult {
        Ok(FA_FONTS.as_object().unwrap().iter()
           .map(|x| {
               let search_terms : Option<Vec<&str>> =
                   x.1["search"]["terms"].as_array()
                   .map(|terms| {
                       terms.iter()
                           .filter(|x| x.is_string())
                           .map(|x| x.as_str().unwrap())
                           .collect()
                   });
               Item {
                   title: x.1["label"].as_str().unwrap().into(),
                   subtitle: search_terms.as_ref().map(|terms| terms.join(", ")),
                   icon: Some(Icon::FontAwesome(x.0.clone())),
                   badge: Some(format!("0x{}", x.1["unicode"].as_str().unwrap())),
                   data: Some(x.0.clone()),
                   search_str: search_terms.as_ref().map(|terms| x.0.clone() + " " + &terms.join(" ")),
                   .. Item::default()
               }
           }).collect())
    }
}

pub fn get(_: &Config) -> Item {
    Item {
        title: "FontAwesome".into(),
        icon: Some(Icon::FontAwesome("font-awesome".into())),
        action: Some(Arc::new(FontAwesome{})),
        .. Item::default()
    }
}
