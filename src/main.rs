use core::panic;
use std::env;
use std::io;
use std::process;

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let mut i = 0;
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
    let Some(c) = input_line.chars().next() else {
        return false;
    };
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
            if matches!(chars.next(), Some((_, 'd')) | Some((_, 'w'))) {
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
        Some(_) => Some(&pattern[..1]),
        None => None,
    }
}

fn match_simple_pattern(input_line: &str, pattern: &str) -> bool {
    input_line
        .chars()
        .next()
        .map(|c| match pattern {
            pat if pat.chars().count() == 1 => pat.starts_with(c),
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

    process::exit((!match_simple_pattern(&input_line, &pattern)).into())
}

#[test]
fn match_single_character() {
    let pattern = "a";
    assert!(match_pattern("apple", pattern));
    assert!(!match_pattern("dog", pattern));
}

#[test]
fn match_digit() {
    let pattern = r"\d";
    assert!(match_pattern("apple123", pattern));
    assert!(!match_pattern("apple", pattern));
}

#[test]
fn match_alphanumeric() {
    let pattern = r"\w";
    assert!(match_pattern("foo101", pattern));
    assert!(!match_pattern("$!?", pattern));
}

#[test]
fn match_character_group() {
    let pattern = "[abc]";
    assert!(match_pattern("apple", pattern));
    assert!(!match_pattern("dog", pattern));
}

#[test]
fn match_negative_character_group() {
    let pattern = "[^abc]";
    assert!(match_pattern("dog", pattern));
    assert!(!match_pattern("cab", pattern));
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
fn match_complex_pattern() {
    let pattern = r"a[bcd]\d[^xyz]\w";
    assert!(match_pattern("For example, ac0fl is a match", pattern));
    assert!(!match_pattern("This sentence has no match", pattern));
}
