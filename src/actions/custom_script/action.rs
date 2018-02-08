extern crate serde_json;
extern crate shlex;

use std;
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
        self.output_to_items(cmd.output()?)
    }

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
