/*
* @Author: BlahGeek
* @Date:   2017-06-17
* @Last Modified by:   BlahGeek
* @Last Modified time: 2018-03-23
*/

extern crate url;
extern crate reqwest;
extern crate serde_json;

use self::url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET, EncodeSet};

use std::sync::Arc;
use std::io::Read;

use mcore::action::{Action, ActionResult};
use mcore::item::{Item, Icon};
use mcore::config::Config;
use actions::utils::open;

#[derive(Clone)]
struct DefaultPlusEncodeSet {}

impl EncodeSet for DefaultPlusEncodeSet {
    fn contains(&self, byte: u8) -> bool {
        DEFAULT_ENCODE_SET.contains(byte) || byte == '+' as u8
    }
}

const DEFAULT_PLUS_ENCODE_SET: DefaultPlusEncodeSet = DefaultPlusEncodeSet{};

#[derive(Deserialize, Clone)]
struct SearchEngine {
    /// Name of the search engine
    name: String,
    /// The URL of the target, replace %s with search text
    address: String,
    /// The URL for suggestion, open search protocol
    suggestion_url: Option<String>,
}


impl Action for SearchEngine {
    fn runnable_arg(&self) -> bool { true }
    fn runnable_arg_realtime(&self) -> bool { self.suggestion_url.is_some() }
    fn runnable_arg_realtime_is_suggestion(&self) -> bool { true }

    fn suggest_arg_scope(&self) -> Option<&str> { Some(&self.name) }

    fn run_arg(&self, text: &str) -> ActionResult {
        let text = utf8_percent_encode(text, DEFAULT_PLUS_ENCODE_SET).to_string();
        let url = self.address.replace("%s", &text);
        open::that(&url)?;
        Ok(Vec::new())
    }

    fn run_arg_realtime(&self, text: &str) -> ActionResult {
        let text = utf8_percent_encode(text, DEFAULT_PLUS_ENCODE_SET).to_string();
        let url = self.suggestion_url.as_ref().unwrap().replace("%s", &text);

        let mut result = String::new();
        reqwest::get(&url)?.read_to_string(&mut result)?;
        let result : serde_json::Value = serde_json::from_str(&result)?;

        let suggestions = match result.get(1) {
            Some(&serde_json::Value::Array(ref arr)) => arr.clone(),
            _ => Vec::new(),
        };
        let suggestions_desc = match result.get(2) {
            Some(&serde_json::Value::Array(ref arr)) => arr.clone(),
            _ => vec![serde_json::Value::String(String::new()); suggestions.len()],
        };

        Ok(
        suggestions.iter().zip(suggestions_desc.iter())
            .filter(|&(a, b)| a.is_string() && b.is_string())
            .map(|(a, b)| {
                let mut item = Item::new_text_item(a.as_str().unwrap());
                item.subtitle = Some(b.as_str().unwrap().into());
                item.icon = Some(Icon::Character{ch: '', font: "FontAwesome".into()});
                item
            })
            .collect::<Vec<Item>>()
        )
    }

}

pub fn get(config: &Config) -> Vec<Item> {
    let sites = config.get::<Vec<SearchEngine>>(&["search_engine", "sites"]).unwrap();
    sites.into_iter()
        .map(|site| {
            debug!("Load search engine: {} = {} ({:?})", site.name, site.address, site.suggestion_url);
            let mut item = Item::new(&site.name);
            item.badge = Some("Search Engine".into());
            item.priority = -10;
            item.icon = Some(Icon::Character{ch: '', font: "FontAwesome".into()});
            item.action = Some(Arc::new(Box::new(site)));
            item
        })
        .collect()
}
