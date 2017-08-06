/*
* @Author: BlahGeek
* @Date:   2017-06-17
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-08-06
*/

extern crate url;
extern crate reqwest;
extern crate serde_json;

use self::url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use toml;

use std::sync::Arc;
use std::io::Read;

use mcore::action::{Action, ActionResult, ActionArg};
use mcore::item::{Item, Icon};
use actions::utils::open;

#[derive(Clone)]
pub struct SearchEngine {
    /// Name of the search engine
    name: String,
    /// The URL of the target, replace %s with search text
    address: String,
    /// The URL for suggestion, open search protocol
    suggestion_url: Option<String>,
}


impl Action for SearchEngine {
    fn get_item(&self) -> Item {
        let mut item = Item::new(&self.name);
        item.badge = Some("Search Engine".into());
        item.priority = -10;
        item.icon = Some(Icon::Character{ch: '', font: "FontAwesome".into()});
        item
    }

    fn accept_text(&self) -> bool { true }
    fn accept_text_realtime(&self) -> bool { self.suggestion_url.is_some() }

    fn run_text(&self, text: &str) -> ActionResult {
        let text = utf8_percent_encode(text, DEFAULT_ENCODE_SET).to_string();
        let url = self.address.replace("%s", &text);
        info!("open: {}", url);
        open::that(&url)?;
        Ok(Vec::new())
    }

    fn run_text_realtime(&self, text: &str) -> ActionResult {
        let url = self.suggestion_url.as_ref().unwrap().replace("%s", text);

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
                item.action = Some(Arc::new(Box::new(self.clone())));
                item.action_arg = ActionArg::Text(item.title.clone());
                item
            })
            .collect::<Vec<Item>>()
        )
    }

}

#[derive(Deserialize)]
struct ConfigSite {
    name: String,
    address: String,
    suggestion_url: Option<String>,
}

#[derive(Deserialize)]
struct Config {
    sites: Vec<ConfigSite>,
}

impl SearchEngine {
    pub fn get_all(config: toml::Value) -> Vec<SearchEngine> {
        let config = config.try_into::<Config>();
        match config {
            Ok(config) =>
                config.sites.into_iter()
                .map(|site| {
                    debug!("Load search engine: {} = {} ({:?})", site.name, site.address, site.suggestion_url);
                    SearchEngine {
                        name: site.name,
                        address: site.address,
                        suggestion_url: site.suggestion_url,
                    }
                })
                .collect(),
            Err(error) => {
                warn!("Error loading search engine sites: {}", error);
                vec![]
            }
        }
    }
}
