/*
* @Author: BlahGeek
* @Date:   2017-06-15
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-07-15
*/

extern crate htmlescape;
use self::htmlescape::encode_minimal;

use std::fmt;
use mcore::item::{Item, ItemData};
use mcore::context::Context;

pub fn format_fit_to_line(text: &str, line_width: i32) -> (String, i32) {
    let mut ret = String::new();
    let mut size = 0;

    if line_width == 0 {
        return (ret, size);
    }

    for c in text.chars() {
        size += 1;
        if size < line_width {
            ret.push(if c == '\n' { ' ' } else { c });
        } else {
            ret.push('…');
            break;
        }
    }

    (ret, size)
}

pub fn format_item(ctx: &Context, item: &Item, line_width: i32) -> String {
    let mut available_width = line_width;
    let mut righttext = String::new();

    if let Some(ref badge) = item.badge {
        righttext = fmt::format(format_args!("[{}] ", badge));
    }
    available_width -= righttext.len() as i32;

    if ctx.selectable_with_text(item) {
        righttext += "T";
    } else if ctx.selectable(item) {
        righttext += "↵";
    } else {
        righttext += " ";
    }

    if let Some(ref action) = item.action {
        if action.should_return_items() {
            righttext += "→";
        } else {
            righttext += "⊙";
        }
    } else {
        righttext += " ";
    }
    available_width -= 2;

    available_width -= 2; // extra padding

    let mut ret = String::new();
    ret.reserve((line_width * 2) as usize);

    ret += "<b>";
    let (title_str, title_str_len) = format_fit_to_line(&item.title, available_width - 1);
    ret += &encode_minimal(&title_str);
    available_width -= title_str_len;
    ret += "</b> ";
    available_width -= 1;

    if available_width > 0 {
        if let Some(ref subtitle) = item.subtitle {
            ret += "<i>";
            let (subtitle_str, subtitle_str_len) = format_fit_to_line(subtitle, available_width);
            ret += &encode_minimal(&subtitle_str);
            available_width -= subtitle_str_len;
            ret += "</i>";
        }
    }

    while available_width > 0 {
        available_width -= 1;
        ret.push(' ');
    }
    ret += "  ";
    ret += &encode_minimal(&righttext);
    ret += "\n";

    ret
}

pub fn format_reference_info(data: &ItemData, line_width: i32) -> String {
    let mut ret = String::from("<u>QuickSend:</u>\n");
    ret.push('\n');

    ret += "<small>";
    match data {
        &ItemData::Text(ref text) => {
            ret += "Text = ";
            let (text_str, _) = format_fit_to_line(text, line_width * 5);
            ret += &encode_minimal(&text_str);
        },
        &ItemData::Path(ref path) => {
            if let Some(path_str) = path.to_str() {
                ret += "Path = ";
                ret += &encode_minimal(path_str);
            }
        },
    }
    ret += "</small>";

    ret
}
