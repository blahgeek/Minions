use std;
use std::path::Path;

use mcore::item::Icon;

pub fn parse_icon(text: &str, script_dir: &std::path::Path) -> Option<Icon> {
    let parts: Vec<&str> = text.splitn(2, ":").collect();
    if parts.len() < 2 {
        None
    } else if parts[0] == "gtk" {
        Some(Icon::GtkName(parts[1].into()))
    } else if parts[0] == "file" {
        if parts[1].starts_with('/') {
            Some(Icon::File(Path::new(parts[1]).to_path_buf()))
        } else {
            Some(Icon::File(script_dir.join(Path::new(parts[1]))))
        }
    } else if parts[0] == "character" {
        let subparts : Vec<&str> = parts[1].splitn(2, ":").collect();
        if subparts.len() < 2 {
            None
        } else {
            let mut ch : String = subparts[1].into();
            if let Some(ch) = ch.pop() {
                Some(Icon::Character{ch: ch, font: subparts[0].into()})
            } else {
                None
            }
        }
    } else {
        None
    }
}

