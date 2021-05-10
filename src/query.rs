use inflector::cases::camelcase::*;
use inflector::cases::kebabcase::*;
use inflector::cases::pascalcase::*;
use inflector::cases::screamingsnakecase::*;
use inflector::cases::snakecase::*;
use inflector::cases::traincase::*;

pub enum Query {
    Substring(String, String),
    Regex(regex::Regex, String),
    Subvert(Vec<String>, Vec<String>),
}

pub fn substring(old: &str, new: &str) -> Query {
    Query::Substring(old.to_string(), new.to_string())
}

pub fn from_regex(re: regex::Regex, replacement: &str) -> Query {
    Query::Regex(re, replacement.to_string())
}

fn to_ugly_case(input: &str) -> String {
    return to_train_case(input).replace("-", "_");
}

pub fn subvert(pattern: &str, replacement: &str) -> Query {
    let mut patterns: Vec<String> = vec![];
    let mut replacements: Vec<String> = vec![];
    for func in &[
        to_camel_case,
        to_kebab_case,
        to_pascal_case,
        to_screaming_snake_case,
        to_snake_case,
        to_train_case,
        to_ugly_case,
    ] {
        patterns.push(func(pattern));
        replacements.push(func(replacement));
    }
    Query::Subvert(patterns, replacements)
}
