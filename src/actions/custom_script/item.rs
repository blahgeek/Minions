use std::path::Path;
use std::sync::Arc;

use crate::mcore::item::Item;

use super::action::ScriptAction;
use super::parser::parse_icon;

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScriptOutputFormat {
    Json,
    EscapedText,
    PlainText,
}

#[derive(Deserialize, Clone)]
#[serde(default = "ScriptItem::default")]
pub struct ScriptItem {
    pub title: String,
    pub subtitle: Option<String>,
    pub badge: Option<String>,
    pub icon: Option<String>,
    pub data: Option<String>,
    pub priority: i32,

    pub action: Option<String>,
    pub action_output_format: ScriptOutputFormat,

    pub action_run_bare: bool,
    pub action_run_arg: bool,
    pub action_run_realtime: bool,

    pub action_suggest_arg_scope: Option<String>,

    pub requirements: Vec<String>,
}

impl ScriptItem {
    pub fn default() -> Self {
        ScriptItem {
            title: "".into(),
            subtitle: None,
            badge: None,
            icon: None,
            data: None,
            priority: -20,
            action: None,
            action_output_format: ScriptOutputFormat::Json,
            action_run_bare: true,
            action_run_arg: false,
            action_run_realtime: true,
            action_suggest_arg_scope: None,
            requirements: Vec::new(),
        }
    }

    pub fn into_item(self, script_dir: &Path) -> Item {

        let action : Option<ScriptAction> =
            match self.action {
                None => None,
                Some(action) =>
                    Some( ScriptAction {
                        script_dir: script_dir.to_path_buf(),
                        action: action,
                        action_output_format: self.action_output_format,
                        action_run_bare: self.action_run_bare,
                        action_run_arg: self.action_run_arg,
                        action_run_realtime: self.action_run_realtime,
                        action_suggest_arg_scope: self.action_suggest_arg_scope,
                    } ),
            };

        let icon =
            match self.icon {
                Some(ref s) => parse_icon(&s, script_dir),
                None => None,
            };

        Item {
            title: self.title,
            subtitle: self.subtitle,
            icon: icon,
            badge: self.badge,
            priority: self.priority,
            data: self.data,
            search_str: None,
            action: match action {
                Some(action) => Some(Arc::new(action)),
                None => None,
            },
        }

    }
}
