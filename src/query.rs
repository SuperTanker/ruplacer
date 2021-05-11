use inflector::cases::camelcase::*;
use inflector::cases::kebabcase::*;
use inflector::cases::pascalcase::*;
use inflector::cases::screamingsnakecase::*;
use inflector::cases::snakecase::*;
use inflector::cases::traincase::*;

pub enum Query {
    Substring(String, String),
    Regex(regex::Regex, String),
    // A list like [(foo_bar, spam_eggs), (FooBar, SpamEggs) ...)]
    Subvert(Vec<(String, String)>),
}

pub fn substring(old: &str, new: &str) -> Query {
    Query::Substring(old.to_string(), new.to_string())
}

pub fn from_regex(re: regex::Regex, replacement: &str) -> Query {
    Query::Regex(re, replacement.to_string())
}

pub fn subvert(pattern: &str, replacement: &str) -> Query {
    fn to_ugly_case(input: &str) -> String {
        to_train_case(input).replace("-", "_")
    }

    let mut items = vec![];
    for func in &[
        to_camel_case,
        to_kebab_case,
        to_pascal_case,
        to_screaming_snake_case,
        to_snake_case,
        to_train_case,
        to_ugly_case,
    ] {
        items.push((func(pattern), func(replacement)));
    }
    Query::Subvert(items)
}
