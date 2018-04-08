/*
* @Author: BlahGeek
* @Date:   2017-06-17
* @Last Modified by:   BlahGeek
* @Last Modified time: 2018-04-08
*/

extern crate url;
extern crate reqwest;
extern crate serde_json;

use self::url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET, EncodeSet};

use std::sync::Arc;
use std::sync::Mutex;
use std::path::Path;

use mcore::action::{Action, ActionResult};
use mcore::item::{Item, Icon};
use mcore::config::Config;
use mcore::errors::*;
use actions::utils::open;
use actions::custom_script::parser::parse_icon;

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
    /// Icon
    icon: Option<String>,

    #[serde(skip, default="default_suggestion_client")]
    suggestion_client: Arc<Mutex<reqwest::Client>>,

    #[serde(skip)]
    icon_parsed: Option<Icon>,
}

fn default_suggestion_client() -> Arc<Mutex<reqwest::Client>> {
    Arc::new(Mutex::new(reqwest::Client::new()))
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

        let result = if let Ok(client) = self.suggestion_client.try_lock() {
            client.get(&url)
                .send().map_err(|e| Error::with_chain(e, "Suggestion request send failed"))?
                .text().map_err(|e| Error::with_chain(e, "Suggestion reply decode failed"))?
        } else {
            warn!("Unable to use shared reqwest client!");
            reqwest::get(&url).map_err(|e| Error::with_chain(e, "Suggestion request send failed"))?
                .text().map_err(|e| Error::with_chain(e, "Suggestion reply decode failed"))?
        };

        let result : serde_json::Value =
            serde_json::from_str(&result).map_err(|e| Error::with_chain(e, "Suggestion reply parse failed"))?;
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
                item.icon = self.icon_parsed.clone()
                    .or(Some(Icon::FontAwesome("search".into())));
                item
            })
            .collect::<Vec<Item>>()
        )
    }

}

pub fn get(config: &Config) -> Vec<Item> {
    let sites = config.get::<Vec<SearchEngine>>(&["search_engine", "sites"]).unwrap();
    sites.into_iter()
        .map(|mut site| {
            site.icon_parsed = site.icon.clone().and_then(
                |x| parse_icon(&x, Path::new(".")));
            site
        })
        .map(|site| {
            debug!("Load search engine: {} = {} ({:?})", site.name, site.address, site.suggestion_url);
            let mut item = Item::new(&site.name);
            item.badge = Some("Search Engine".into());
            item.priority = -10;
            item.icon = site.icon_parsed.clone()
                .or(Some(Icon::FontAwesome("search".into())));
            item.action = Some(Arc::new(Box::new(site)));
            item
        })
        .collect()
}
