/*
* @Author: BlahGeek
* @Date:   2017-06-24
* @Last Modified by:   BlahGeek
* @Last Modified time: 2018-04-08
*/

extern crate url;
extern crate reqwest;
extern crate crypto;
extern crate serde_json;

use self::crypto::digest::Digest;
use self::url::form_urlencoded;

use std::io::Read;
use std::sync::Arc;
use mcore::action::{Action, ActionResult};
use mcore::item::{Item, Icon};
use mcore::config::Config;
use mcore::errors::*;

struct Youdao {}

// yes, here are both app key and secret, I dont care
static APP_KEY: &'static str = "259f2733d8e07293";
static APP_SECRET: &'static str = "36pNoOHoQsjm48njBzrdgyY2Y52moDRp";

#[derive(Deserialize, Clone)]
struct YoudaoResultBasic {
    phonetic: Option<String>,
    explains: Vec<String>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Clone)]
struct YoudaoResult {
    errorCode: String,
    query: String,
    translation: Vec<String>,

    basic: Option<YoudaoResultBasic>,
}

impl Action for Youdao {

    fn runnable_arg (&self) -> bool { true }

    fn run_arg (&self, text: &str) -> ActionResult {
        let salt = "WTF";
        let mut hash = crypto::md5::Md5::new();
        hash.input(APP_KEY.as_bytes());
        hash.input(text.as_bytes());
        hash.input(salt.as_bytes());
        hash.input(APP_SECRET.as_bytes());
        let encoded: String = form_urlencoded::Serializer::new(String::new())
            .append_pair("q", &text)
            .append_pair("from", "auto")
            .append_pair("to", "auto")
            .append_pair("appKey", APP_KEY)
            .append_pair("salt", &salt)
            .append_pair("sign", &hash.result_str())
            .finish();

        let url = String::new() + "https://openapi.youdao.com/api?" + &encoded;
        trace!("Youdao request url: {}", url);

        let mut result = String::new();
        reqwest::get(&url).map_err(|e| Error::with_chain(e, "Failed to perform HTTP request"))?
            .read_to_string(&mut result).map_err(|e| Error::with_chain(e, "Failed to read request reply"))?;

        let result : YoudaoResult = serde_json::from_str(&result)
            .map_err(|e| Error::with_chain(e, "Failed parsing JSON"))?;
        if result.errorCode != "0" || result.translation.len() == 0 {
            bail!("Invalid youdao API return code {}", result.errorCode);
        }

        let mut main_text = String::new();
        if let Some(ref basic) = result.basic {
            if let Some(ref phonetic) = basic.phonetic {
                main_text = format!("[{}] ", phonetic);
            }
        }
        main_text += &result.translation[0];

        let mut main_item = Item::new_text_item(&main_text);
        main_item.subtitle = Some(result.query);

        let mut ret = vec![main_item];

        if let Some(basic) = result.basic {
            for explain in basic.explains {
                ret.push(Item::new_text_item(&explain));
            }
        }

        Ok(ret)
    }
}

pub fn get(_: &Config) -> Item {
    let mut item = Item::new("Youdao Translate");
    item.badge = Some("Translate".into());
    item.priority = -5;
    item.icon = Some(Icon::FontAwesome("globe".into()));
    item.action = Some(Arc::new(Youdao{}));
    item
}

#[test]
fn test_youdao() {
    let youdao = Youdao {};
    let res = youdao.run_arg("hello");
    if let Err(error) = res {
        println!("{}", error);
    }
}
