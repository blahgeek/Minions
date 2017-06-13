/*
* @Author: BlahGeek
* @Date:   2017-06-13
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-14
*/

use std::fmt;
use std::rc::Rc;
use std::io::Write;
use std::io::Read;
use std::error::Error;
use std::process::{Command, Stdio};

use mcore::context::Context;
use mcore::item::Item;

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

impl MinionsApp {

    fn rofi_enter_text(&mut self, item: Rc<Item>) -> Result<State, Box<Error>> {
        let mut cmd = Command::new("rofi");
        let prompt = format!("{}> ", item.title);
        cmd.stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .arg("-dmenu")
           .arg("-p").arg(&prompt)
           .arg("-format").arg("f");
        println!("Executing: {:?}", cmd);

        let mut child = cmd.spawn()?;

        let status = child.wait()?.code().unwrap();
        let mut stdout_str = String::new();
        child.stdout.unwrap().read_to_string(&mut stdout_str)?;
        println!("Rofi output: {:?}", stdout_str);

        Ok (match status {
            0 => { // enter
                self.ctx.select_with_text(item, &stdout_str);
                State::Filtering(-1, String::new())
            },
            1 => { // esc
                println!("Return to filter mode");
                State::Filtering(-1, String::new())
            },
            _ => {
                panic!("Unexpected rofi return code");
                State::Exiting
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
           .arg("-no-custom")
           .arg("-p").arg(&prompt)
           .arg("-format").arg("i|f")
           .arg("-selected-row").arg(select_idx.to_string())
           .arg("-filter").arg(filter_str.to_string())
           .arg("-kb-custom-1").arg("space")
           .arg("-kb-row-tab").arg("") // disable default Tab
           .arg("-kb-custom-2").arg("Tab");
        println!("Executing: {:?}", cmd);

        let mut child = cmd.spawn()?;

        for item in self.ctx.list_items.iter() {
            if let Some(ref mut stdin) = child.stdin {
                stdin.write_fmt(format_args!("{}\n", item.title))?;
            }
        }
        let status = child.wait()?.code().unwrap();

        if status == 1 {
            // exited without selection
            return Ok(State::Exiting)
        }

        let mut stdout_str = String::new();
        child.stdout.unwrap().read_to_string(&mut stdout_str)?;
        println!("Rofi output: {:?}", stdout_str);

        let mut stdout_str: Vec<&str> = stdout_str.splitn(2, '|').collect();

        let filter_str = stdout_str.pop().unwrap().trim();
        let selected_idx: i32 = stdout_str.pop().unwrap().parse()?;
        let selected_item = self.ctx.list_items[selected_idx as usize].clone();

        Ok( match status {
            10 => { // space
                if self.ctx.selectable_with_text(&selected_item) {
                    State::EnteringText(selected_item)
                } else {
                    println!("Item {} not selectable with text", selected_item.title);
                    State::Filtering(selected_idx, filter_str.into())
                }
            },
            11 => { // tab
                if self.ctx.quicksend_able(&selected_item) {
                    self.ctx.quicksend(selected_item)?;
                    State::Filtering(-1, String::new())
                } else {
                    println!("Item {} not quicksend-able", selected_item.title);
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
                    println!("Item {} not selectable", selected_item.title);
                    State::Filtering(selected_idx, filter_str.into())
                }
            },
            _ => {
                panic!("Unexpected rofi return code");
            }
        } )

    }

    pub fn run_loop(&mut self) {
        loop {
            if self.ctx.list_items.len() == 0 {
                println!("No listing items");
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
                    println!("Error: {}", err);
                    break;
                },
                Ok(State::Exiting) => {
                    println!("Exit!");
                    break;
                },
                Ok(s) => {
                    self.state = s;
                }
            };
        }
    }


    pub fn new() -> MinionsApp {
        MinionsApp {
            ctx: Context::new(),
            state: State::Filtering(-1, String::new()),
        }
    }

}
