/*
* @Author: BlahGeek
* @Date:   2017-06-15
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-15
*/

use std::fmt;
use mcore::item::Item;
use mcore::context::Context;

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
    for c in item.title.chars() {
        if c == '\n' { continue; }
        available_width -= 1;
        if available_width > 1 {
            ret.push(c);
        } else {
            ret.push('…');
            break;
        }
    }
    ret += "</b> ";
    available_width -= 1;

    if available_width > 0 {
        if let Some(ref subtitle) = item.subtitle {
            ret += "<i>";
            for c in subtitle.chars() {
                if c == '\n' { continue; }
                available_width -= 1;
                if available_width > 0 {
                    ret.push(c);
                } else {
                    ret.push('…');
                    break;
                }
            }
            ret += "</i>";
        }
    }

    while available_width > 0 {
        available_width -= 1;
        ret.push(' ');
    }
    ret += "  ";
    ret += &righttext;
    ret += "\n";

    ret
}
