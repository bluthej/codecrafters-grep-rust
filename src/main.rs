use std::env;
use std::io;
use std::process;

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    match pattern {
        pat if pat.chars().count() == 1 => input_line.contains(pattern),
        r"\d" => input_line.chars().any(|c| c.is_ascii_digit()),
        r"\w" => input_line.chars().any(|c| c.is_ascii_alphanumeric()),
        pat if pat.starts_with('[') && pat.ends_with(']') => {
            let pat = pat
                .strip_prefix('[')
                .expect("`pat` starts with [")
                .strip_suffix(']')
                .expect("`pat` ends with ]");
            if pat.starts_with('^') {
                let pat = pat.strip_prefix('^').expect("`pat` starts with ^");
                !input_line.chars().any(|c| pat.contains(c))
            } else {
                input_line.chars().any(|c| pat.contains(c))
            }
        }
        _ => panic!("Unhandled pattern: {}", pattern),
    }
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

#[test]
fn match_single_character() {
    assert!(match_pattern("apple", "a"));
    assert!(!match_pattern("apple", "c"));
}

#[test]
fn match_digit() {
    assert!(match_pattern("apple123", r"\d"));
    assert!(!match_pattern("apple", r"\d"));
}

#[test]
fn match_alphanumeric() {
    assert!(match_pattern("foo101", r"\w"));
    assert!(!match_pattern("$!?", r"\w"));
}
