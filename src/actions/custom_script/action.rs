extern crate serde_json;
extern crate shlex;

use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

use mcore::action::{ActionResult, Action};
use actions::ActionError;

use super::item::{ScriptOutputFormat, ScriptItem};

pub struct ScriptAction {
    pub script_dir: PathBuf,

    pub action: String,
    pub action_output_format: ScriptOutputFormat,

    pub action_run_bare: bool,
    pub action_run_arg: bool,
    pub action_run_realtime: bool,
}

impl Action for ScriptAction {
    fn runnable_bare(&self) -> bool { self.action_run_bare }
    fn runnable_arg(&self) -> bool { self.action_run_arg }
    fn runnable_arg_realtime(&self) -> bool { self.action_run_realtime }

    fn run_bare (&self) -> ActionResult {
        self.run_action(None, "bare")
    }

    fn run_arg(&self, text: &str) -> ActionResult {
        self.run_action(Some(text), "text")
    }

    fn run_arg_realtime(&self, text: &str) -> ActionResult {
        self.run_action(Some(text), "realtime")
    }
}


fn parse_json (output: &[u8]) -> Result<Vec<ScriptItem>, Box<Error + Send + Sync>> {
    Ok(serde_json::from_slice(output)?)
}

fn parse_escaped_test (output: &[u8]) -> Result<Vec<ScriptItem>, Box<Error + Send + Sync>> {
    let mut ret : Vec<ScriptItem> = Vec::new();
    for item_output in output.split(|x| *x == 0u8) {
        let mut item_json = serde_json::map::Map::<String, serde_json::Value>::new();
        for line in item_output.split(|x| *x == 1u8) {
            let line = String::from_utf8(line.to_vec())?;
            let parts: Vec<&str> = line.as_str().trim().splitn(2, ':').collect();
            if parts.len() == 2 {
                item_json.insert(parts[0].into(), serde_json::Value::String(parts[1].into()));
            }
        }
        ret.push(serde_json::from_value(serde_json::Value::Object(item_json))?);
    }
    Ok(ret)
}

impl ScriptAction {

    fn run_action (&self, arg: Option<&str>, typ: &str) -> ActionResult {
        // TODO: support some special commands, like copy, open, etc.
        let cmdline = shlex::split(&self.action);
        if cmdline.is_none() {
            return Err(Box::new(ActionError::new("Invalid action command")))
        }

        let cmdline = cmdline.unwrap();
        let mut cmd = Command::new(&cmdline[0]);
        if cmdline.len() > 1 {
            cmd.args(&cmdline[1..]);
        }
        if let Some(arg) = arg {
            cmd.arg(arg);
        }
        cmd.current_dir(&self.script_dir);
        cmd.env("MINIONS_RUN_TYPE", typ);
        debug!("Running script action: {:?}", cmd);

        let output = cmd.output()?.stdout;
        let items = match self.action_output_format {
            ScriptOutputFormat::Json => parse_json(&output),
            ScriptOutputFormat::EscapedText => parse_escaped_test(&output),
        }?;

        Ok(items.into_iter().map(|x| x.into_item(&self.script_dir)).collect())
    }
}
