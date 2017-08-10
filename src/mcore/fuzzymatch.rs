/*
* @Author: BlahGeek
* @Date:   2017-04-19
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-08-10
*/


pub fn fuzzymatch(text: &str, pattern: &str, casesensitive: bool) -> i32 {
    if pattern.len() == 0 { return 0; }

    let mut text_iter = text.chars().peekable();
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
    use mcore::fuzzymatch::fuzzymatch;
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
        assert!(fuzzymatch("你好 世界", "世界", false) > 0);
        assert!(fuzzymatch("你好 世界", "你世", false) > 0);
        assert!(fuzzymatch("", "hw", false) == 0);
    }
}
