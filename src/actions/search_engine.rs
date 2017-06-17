/*
* @Author: BlahGeek
* @Date:   2017-06-17
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-17
*/

extern crate url;

use self::url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

use std::error::Error;
use std::process::Command;

use mcore::action::Action;
use mcore::item::Item;

pub struct SearchEngine {
    /// Name of the search engine
    name: String,
    /// The URL of the target, replace %s with search text
    address: String,
}


impl Action for SearchEngine {
    fn get_item(&self) -> Item {
        let mut item = Item::new(&self.name);
        item.badge = Some("Search Engine".into());
        item
    }

    fn accept_text(&self) -> bool { true }

    fn run_text(&self, text: &str) -> Result<Vec<Item>, Box<Error>> {
        let text = utf8_percent_encode(text, DEFAULT_ENCODE_SET).to_string();
        let url = self.address.replace("%s", &text);
        println!("xdg-open: {}", url);
        Command::new("xdg-open").arg(&url).output()?;
        Ok(Vec::new())
    }
}

impl SearchEngine {
    pub fn get_all() -> Vec<SearchEngine> {
        vec![
            SearchEngine {
                name: "Google".into(),
                address: "https://www.google.com/search?q=%s".into(),
            },
            SearchEngine {
                name: "Bing".into(),
                address: "https://www.bing.com/search?q=%s".into(),
            },
            SearchEngine {
                name: "DuckDuckGo".into(),
                address: "https://duckduckgo.com/?q=%s".into(),
            },
            SearchEngine {
                name: "Wikipedia".into(),
                address: "https://en.wikipedia.org/wiki/Special:Search?search=%s".into(),
            },
        ]
    }
}
