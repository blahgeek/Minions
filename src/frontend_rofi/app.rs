/*
* @Author: BlahGeek
* @Date:   2017-06-13
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-18
*/

use toml;

use std::rc::Rc;
use std::io::Write;
use std::io::Read;
use std::error::Error;
use std::process::{Command, Stdio};

use mcore::context::Context;
use mcore::item::Item;

use frontend_rofi::utils;

#[derive(Clone)]
enum State {
    Filtering(i32, String),
    EnteringText(Rc<Item>),
    Exiting,
}

pub struct MinionsApp {
    ctx: Context,

    state: State,
}

static ROFI_WIDTH: i32 = 120;

impl MinionsApp {

    fn rofi_enter_text(&mut self, item: Rc<Item>) -> Result<State, Box<Error>> {
        let mut cmd = Command::new("rofi");
        let prompt = format!("{}> ", item.title);
        cmd.stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .arg("-dmenu")
           .arg("-width").arg((-ROFI_WIDTH-2).to_string())
           .arg("-p").arg(&prompt)
           .arg("-format").arg("f");
        debug!("Executing: {:?}", cmd);

        let mut child = cmd.spawn()?;

        let status = child.wait()?.code().unwrap();
        let mut stdout_str = String::new();
        child.stdout.unwrap().read_to_string(&mut stdout_str)?;
        let stdout_str = stdout_str.as_str().trim();

        Ok (match status {
            0 => { // enter
                self.ctx.select_with_text(item, &stdout_str)?;
                State::Filtering(-1, String::new())
            },
            1 => { // esc
                debug!("Return to filter mode");
                State::Filtering(-1, String::new())
            },
            _ => {
                panic!("Unexpected rofi return code");
            }
        })
    }

    fn rofi_filter(&mut self, select_idx: i32, filter_str: &str) -> Result<State, Box<Error>> {
        let mut cmd = Command::new("rofi");

        let prompt = match self.ctx.reference_item {
            None => "Minions: ".into(),
            Some(ref item) => format!("{}> ", item.title),
        };
        cmd.stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .arg("-dmenu")
           .arg("-i")  // case insensitive
           .arg("-no-custom")
           .arg("-markup-rows")
           .arg("-width").arg((-ROFI_WIDTH-2).to_string())
           .arg("-p").arg(&prompt)
           .arg("-format").arg("i|f")
           .arg("-selected-row").arg(select_idx.to_string())
           .arg("-filter").arg(filter_str.to_string())
           .arg("-kb-custom-1").arg("space")
           .arg("-kb-row-tab").arg("") // disable default Tab
           .arg("-kb-custom-2").arg("Tab");
        if let Some(ref item) = self.ctx.reference_item {
            let msg = utils::format_reference_info(item);
            cmd.arg("-mesg").arg(&msg);
        }
        debug!("Executing: {:?}", cmd);

        let mut child = cmd.spawn()?;

        for item in self.ctx.list_items.iter() {
            if let Some(ref mut stdin) = child.stdin {
                let item_str = utils::format_item(&self.ctx, item, ROFI_WIDTH).into_bytes();
                stdin.write(&item_str)?;
            }
        }
        let status = child.wait()?.code().unwrap();

        if status == 1 {
            // exited without selection
            return Ok(State::Exiting)
        }

        let mut stdout_str = String::new();
        child.stdout.unwrap().read_to_string(&mut stdout_str)?;

        let mut stdout_str: Vec<&str> = stdout_str.splitn(2, '|').collect();

        let filter_str = stdout_str.pop().unwrap().trim();
        let selected_idx: i32 = stdout_str.pop().unwrap().parse()?;
        let selected_item = self.ctx.list_items[selected_idx as usize].clone();

        Ok( match status {
            10 => { // space
                if self.ctx.selectable_with_text(&selected_item) {
                    State::EnteringText(selected_item)
                } else {
                    warn!("Item {} not selectable with text", selected_item.title);
                    State::Filtering(selected_idx, filter_str.into())
                }
            },
            11 => { // tab
                if self.ctx.quicksend_able(&selected_item) {
                    self.ctx.quicksend(selected_item)?;
                    State::Filtering(-1, String::new())
                } else {
                    warn!("Item {} not quicksend-able", selected_item.title);
                    State::Filtering(selected_idx, filter_str.into())
                }
            },
            0 => { // enter
                if self.ctx.selectable(&selected_item) {
                    self.ctx.select(selected_item)?;
                    State::Filtering(-1, String::new())
                } else if self.ctx.selectable_with_text(&selected_item) {
                    State::EnteringText(selected_item)
                } else {
                    warn!("Item {} not selectable", selected_item.title);
                    State::Filtering(selected_idx, filter_str.into())
                }
            },
            _ => {
                panic!("Unexpected rofi return code");
            }
        } )

    }

    pub fn run_loop(&mut self) {
        let mut exiting_count = 0;
        loop {
            if self.ctx.list_items.len() == 0 {
                warn!("No listing items");
                break;
            }
            let new_state = match self.state.clone() {
                State::Filtering(select_idx, ref filter_str) => {
                    self.rofi_filter(select_idx, &filter_str)
                },
                State::EnteringText(item) => {
                    self.rofi_enter_text(item)
                },
                _ => {
                    Ok(State::Exiting)
                }
            };
            match new_state {
                Err(err) => {
                    error!("Error: {}", err);
                    break;
                },
                Ok(State::Exiting) => {
                    if exiting_count == 0 {
                        self.ctx.reset();
                        self.state = State::Filtering(-1, String::new());
                        exiting_count += 1;
                    } else {
                        info!("Exit!");
                        break;
                    }
                },
                Ok(s) => {
                    exiting_count = 0;
                    self.state = s;
                }
            };
        }
    }


    pub fn new(config: toml::Value, from_clipboard: bool) -> MinionsApp {
        let mut app = MinionsApp {
            ctx: Context::new(config),
            state: State::Filtering(-1, String::new()),
        };
        if from_clipboard {
            if let Err(error) = app.ctx.quicksend_from_clipboard() {
                warn!("Unable to get content from clipboard: {}", error);
            }
        }
        app
    }

}
