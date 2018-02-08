extern crate serde_json;

use std;
use std::path::PathBuf;
use std::process::Command;

use mcore::action::{ActionResult, Action};
use actions::ActionError;

use super::item::{ScriptOutputFormat, ScriptItem};

pub struct ScriptAction {
    pub script_dir: PathBuf,

    pub action: Vec<String>,
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
        // TODO: support some special commands, like copy, open, etc.
        let mut cmd = Command::new(&self.action[0]);
        cmd.args(&self.action[1..]);
        cmd.current_dir(&self.script_dir);
        debug!("Running script action: {:?}", cmd);
        self.output_to_items(cmd.output()?)
    }

    fn run_arg(&self, text: &str) -> ActionResult {
        let mut cmd = Command::new(&self.action[0]);
        cmd.arg(text);
        cmd.current_dir(&self.script_dir);
        debug!("Running script action (with text): {:?}", cmd);
        self.output_to_items(cmd.output()?)
    }

    fn run_arg_realtime(&self, text: &str) -> ActionResult {
        let mut cmd = Command::new(&self.action[0]);
        cmd.arg(text);
        cmd.current_dir(&self.script_dir);
        cmd.env("MINIONS_RUN_REALTIME", "1");
        debug!("Running script action (with text realtime): {:?}", cmd);
        self.output_to_items(cmd.output()?)
    }
}

impl ScriptAction {
    fn output_to_items(&self, output: std::process::Output) -> ActionResult {
        if !output.status.success() {
            Err(Box::new(ActionError::new("Action execution failed")))
        } else {
            // TODO: match self.script_return_format
            let output = &output.stdout;
            let json_output : Vec<ScriptItem> = serde_json::from_slice(output)?;
            if json_output.len() > 0 {
                Ok(json_output.into_iter()
                   .map(|x| x.into_item(&self.script_dir))
                   .collect())
            } else {
                Err(Box::new(ActionError::new("Empty result from action")))
            }
        }
    }
}
