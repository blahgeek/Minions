/*
* @Author: BlahGeek
* @Date:   2017-04-19
* @Last Modified by:   BlahGeek
* @Last Modified time: 2018-02-15
*/

extern crate pinyin;

use std::iter::Iterator;
use std::collections::VecDeque;
use std::str::Chars;

struct PinyinChars<'a> {
    pyqueue: VecDeque<char>,
    chars: Chars<'a>,
}

impl<'a> Iterator for PinyinChars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        if let Some(c) = self.pyqueue.pop_front() {
            return Some(c)
        }
        if let Some(c) = self.chars.next() {
            if c.is_ascii() {
                Some(c)
            } else {
                let mut s = String::new();
                s.push(c);
                for word in pinyin::lazy_pinyin(&s, &pinyin::Args::new()).into_iter() {
                    for c in word.chars() {
                        self.pyqueue.push_back(c)
                    }
                    self.pyqueue.push_back(' ')
                }
                Some(' ')
            }
        } else {
            None
        }
    }
}

impl<'a> PinyinChars<'a> {
    fn new(s: &'a str) -> PinyinChars<'a> {
        PinyinChars {
            pyqueue: VecDeque::with_capacity(8),
            chars: s.chars(),
        }
    }
}

pub fn fuzzymatch(text: &str, pattern: &str, casesensitive: bool) -> i32 {
    if pattern.len() == 0 { return 0; }

    let pinyin_chars = PinyinChars::new(text);
    let mut text_iter = pinyin_chars.peekable();
    let mut pattern_iter = pattern.chars();

    let mut score = 0;
    let mut firstchar_bonus = 1;

    let mut match_success = false;
    let mut last_text_ch: char = '\u{0}';

    'outer: loop {
        match pattern_iter.next() {
            None => {
                match_success = true;
                break;
            },
            Some(pattern_ch) => {
                let mut skipped_count = 0;
                while let Some(text_ch) = text_iter.next() {
                    if (casesensitive && text_ch != pattern_ch) ||
                       (!casesensitive && text_ch.to_lowercase().next() != pattern_ch.to_lowercase().next() ) {
                        skipped_count += 1;
                        last_text_ch = text_ch;
                    } else {
                        score += 1;
                        if skipped_count == 0 {
                            score += 1;
                        }
                        if text_ch.is_uppercase() || (text_ch.is_alphanumeric() &&
                                                      !last_text_ch.is_alphanumeric()) {
                            firstchar_bonus += 1;
                        }
                        last_text_ch = text_ch;
                        continue 'outer;
                    }
                }
                if let None = text_iter.peek() {
                    break 'outer;
                }
            }
        }
    }

    if match_success {
        score * firstchar_bonus
    } else {
        0
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;

    #[test]
    fn fuzzymatch_test() {
        assert!(fuzzymatch("hello world", "hw", false) > 0);
        assert!(fuzzymatch("hello world", "hw", false) >
                fuzzymatch("hello world", "he", false));
        assert!(fuzzymatch("hello world", "hww", false) == 0);
        assert!(fuzzymatch("Hello World", "hw", false) > 0);
        assert!(fuzzymatch("Hello World", "hw", true) == 0);
        assert!(fuzzymatch("Hello World", "helloworld", false) >
                fuzzymatch("Hello World", "hello", false));
        assert!(fuzzymatch("Hello World", "world", false) > 0);
        // assert!(fuzzymatch("你好 世界", "世界", false) > 0);
        // assert!(fuzzymatch("你好 世界", "你世", false) > 0);
        assert!(fuzzymatch("", "hw", false) == 0);
    }

    #[test]
    fn pinyinchars_test() {
        assert_eq!(PinyinChars::new("你好 world").collect::<Vec<char>>(),
                   &[' ', 'n', 'i', ' ',
                   ' ', 'h', 'a', 'o', ' ',
                   ' ', 'w', 'o', 'r', 'l', 'd']);
        assert!(fuzzymatch("你好 世界", "nhsj", false) > 0);
        assert!(fuzzymatch("你好 世界", "ni", false) > 0);
    }
}
