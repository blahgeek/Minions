/*
* @Author: BlahGeek
* @Date:   2017-06-17
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-27
*/

extern crate url;

use self::url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use toml;

use mcore::action::{Action, ActionResult};
use mcore::item::Item;
use actions::utils::open;

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
        item.priority = -10;
        item
    }

    fn accept_text(&self) -> bool { true }

    fn run_text(&self, text: &str) -> ActionResult {
        let text = utf8_percent_encode(text, DEFAULT_ENCODE_SET).to_string();
        let url = self.address.replace("%s", &text);
        info!("open: {}", url);
        open::that(&url)?;
        Ok(Vec::new())
    }
}

#[derive(Deserialize)]
struct ConfigSite {
    name: String,
    address: String,
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
                    debug!("Load search engine: {} = {}", site.name, site.address);
                    SearchEngine {
                        name: site.name,
                        address: site.address,
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
