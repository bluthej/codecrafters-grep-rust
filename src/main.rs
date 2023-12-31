use core::panic;
use std::env;
use std::io;
use std::process;

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let mut i = 0;
    if let Some(pattern) = pattern.strip_prefix('^') {
        return match_here(input_line, pattern);
    }
    while i <= input_line.len() {
        if match_here(&input_line[i..], pattern) {
            return true;
        }
        i += input_line.chars().next().map(char::len_utf8).unwrap_or(1);
    }
    false
}

fn match_here(input_line: &str, pattern: &str) -> bool {
    let Some(pat) = next_pattern(pattern) else {
        return true;
    };
    match pattern[pat.len()..].chars().next() {
        Some('*') => return match_star(input_line, pat, &pattern[pat.len() + 1..]),
        Some('+') => {
            let Some(c) = input_line.chars().next() else {
                return false;
            };
            let n = c.len_utf8();
            return match_simple_pattern(input_line, pat)
                && match_star(&input_line[n..], pat, &pattern[pat.len() + 1..]);
        }
        Some('?') => {
            if input_line.is_empty() {
                return pattern[pat.len() + 1..].is_empty();
            }
            return match_here(input_line, &[&pat, &pattern[pat.len() + 1..]].concat())
                || match_here(input_line, &pattern[pat.len() + 1..]);
        }
        _ => {}
    }
    let Some(c) = input_line.chars().next() else {
        return pat == "$";
    };
    if pat.starts_with('(') {
        let (Some(bar), Some(rparen)) = (pattern.find('|'), pattern.find(')')) else {
            panic!("Unterminated alternation")
        };
        return match_here(
            input_line,
            &[&pattern[1..bar], &pattern[rparen + 1..]].concat(),
        ) || match_here(
            input_line,
            &[&pattern[bar + 1..rparen], &pattern[rparen + 1..]].concat(),
        );
    }
    if match_simple_pattern(input_line, pat) {
        match_here(&input_line[c.len_utf8()..], &pattern[pat.len()..])
    } else {
        false
    }
}

fn next_pattern(pattern: &str) -> Option<&str> {
    let mut chars = pattern.char_indices();
    match chars.next() {
        Some((_, '\\')) => {
            if matches!(
                chars.next(),
                Some((_, 'd')) | Some((_, 'w')) | Some((_, '\\'))
            ) {
                Some(&pattern[..2])
            } else {
                panic!("Unhandled pattern: {}", pattern)
            }
        }
        Some((_, '[')) => {
            for (n, c) in chars {
                if c == ']' {
                    return Some(&pattern[..n + 1]);
                }
            }
            panic!("Unterminated character group");
        }
        Some((_, '(')) => {
            let (Some(_), Some(rparen)) = (pattern.find('|'), pattern.find(')')) else {
                panic!("Unterminated alternation")
            };
            Some(&pattern[..rparen + 1])
        }
        Some(_) => Some(&pattern[..1]),
        None => None,
    }
}

fn match_star(input_line: &str, pattern: &str, rest: &str) -> bool {
    let mut i = 0;
    while i <= input_line.len() {
        if match_here(&input_line[i..], rest) {
            return true;
        }
        if !match_simple_pattern(&input_line[i..], pattern) {
            break;
        }
        i += input_line.chars().next().map(char::len_utf8).unwrap_or(1);
    }
    false
}

fn match_simple_pattern(input_line: &str, pattern: &str) -> bool {
    input_line
        .chars()
        .next()
        .map(|c| match pattern {
            pat if pat.chars().count() == 1 => pat.starts_with(c) | pat.starts_with('.'),
            r"\\" => c == '\\',
            r"\d" => c.is_ascii_digit(),
            r"\w" => c.is_ascii_alphanumeric(),
            pat if pat.starts_with('[') && pat.ends_with(']') => {
                let pat = pat
                    .strip_prefix('[')
                    .expect("`pat` starts with [")
                    .strip_suffix(']')
                    .expect("`pat` ends with ]");
                if pat.starts_with('^') {
                    let pat = pat.strip_prefix('^').expect("`pat` starts with ^");
                    !pat.contains(c)
                } else {
                    pat.contains(c)
                }
            }
            _ => panic!("Unhandled pattern: {}", pattern),
        })
        .unwrap_or(false)
}

// Usage: echo <input_text> | your_grep.sh -E <pattern>
fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    process::exit((!match_pattern(&input_line, &pattern)).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_single_character() {
        assert!(match_pattern("dog", "d"));
        assert!(!match_pattern("dog", "f"));
    }

    #[test]
    fn match_digit() {
        assert!(match_pattern("123", r"\d"));
        assert!(!match_pattern("apple", r"\d"));
    }

    #[test]
    fn match_alphanumeric() {
        assert!(match_pattern("word", r"\w"));
        assert!(!match_pattern("$!?", r"\w"));
    }

    #[test]
    fn match_character_group() {
        assert!(match_pattern("a", "[abcd]"));
        assert!(!match_pattern("efgh", "[abcd]"));
    }

    #[test]
    fn match_negative_character_group() {
        assert!(match_pattern("apple", "[^xyz]"));
        assert!(!match_pattern("banana", "[^anb]"));
    }

    #[test]
    fn match_empty_pattern() {
        let pattern = "";
        assert!(match_pattern("dog", pattern));
    }

    #[test]
    fn match_empty_input() {
        let pattern = "a";
        assert!(!match_pattern("", pattern));
    }

    #[test]
    fn match_combine_character_classes() {
        assert!(match_pattern("sally has 3 apples", r"\d apple"));
        assert!(!match_pattern("sally has 1 orange", r"\d apple"));
        assert!(match_pattern("sally has 124 apples", r"\d\d\d apples"));
        assert!(!match_pattern("sally has 12 apples", r"\d\\d\\d apples"));
        assert!(match_pattern("sally has 3 dogs", r"\d \w\w\ws"));
        assert!(match_pattern("sally has 4 dogs", r"\d \w\w\ws"));
        assert!(!match_pattern("sally has 1 dog", r"\d \w\w\ws"));
    }

    #[test]
    fn match_start_of_string_anchor() {
        assert!(match_pattern("log", "^log"));
        assert!(!match_pattern("slog", "^log"));
    }

    #[test]
    fn match_end_of_string_anchor() {
        assert!(match_pattern("dog", "dog$"));
        assert!(!match_pattern("dogs", "dog$"));
    }

    #[test]
    fn match_star() {
        assert!(match_pattern("ale", "ap*le"));
        assert!(match_pattern("aple", "ap*le"));
        assert!(match_pattern("apple", "ap*le"));
        assert!(match_pattern("apppppple", "ap*le"));
        assert!(!match_pattern("apple", "ap*la"));
    }

    #[test]
    fn match_plus() {
        assert!(match_pattern("aple", "ap+le"));
        assert!(match_pattern("apple", "ap+le"));
        assert!(match_pattern("apppppple", "ap+le"));
        assert!(!match_pattern("ale", "ap+le"));
    }

    #[test]
    fn match_question_mark() {
        assert!(match_pattern("dog", "dogs?"));
        assert!(match_pattern("dogs", "dogs?"));
        assert!(match_pattern("aple", "app?le"));
        assert!(match_pattern("apple", "app?le"));
        assert!(!match_pattern("cat", "dogs?"));
        assert!(!match_pattern("apple", "apx?le"));
    }

    #[test]
    fn match_dot() {
        assert!(match_pattern("dog", "d.g"));
        assert!(match_pattern("dig", "d.g"));
        assert!(!match_pattern("cog", "d.g"));
    }

    #[test]
    fn match_alternation() {
        assert!(match_pattern("cat", "(cat|dog)"));
        assert!(match_pattern("dog", "(cat|dog)"));
        assert!(!match_pattern("apple", "(cat|dog)"));
    }

    #[test]
    fn match_alternation_with_variable_length_match() {
        assert!(match_pattern(
            "the cat is eating",
            "the (ca+t|dog) is eating"
        ));
        assert!(match_pattern(
            "the caaaaaaaat is eating",
            "the (ca+t|dog) is eating"
        ));
        assert!(match_pattern(
            "the dog is eating",
            "the (ca+t|dog) is eating"
        ));
        assert!(!match_pattern(
            "the bird is eating",
            "the (ca+t|dog) is eating"
        ));
    }

    #[test]
    fn match_question_mark_after_alternation() {
        assert!(match_pattern("", "(cat|bird)?"));
        assert!(match_pattern("cat", "(cat|bird)?"));
        assert!(match_pattern("bird", "(cat|bird)?"));
        assert!(match_pattern("catdog", "(cat|bird)?dog"));
    }
}
