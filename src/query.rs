pub enum Query {
    Substring(String, String),
    Regex(regex::Regex, String),
    Subvert(String, String),
}

pub fn substring(old: &str, new: &str) -> Query {
    Query::Substring(old.to_string(), new.to_string())
}

pub fn from_regex(re: regex::Regex, replacement: &str) -> Query {
    Query::Regex(re, replacement.to_string())
}

pub fn subvert(pattern: &str, replacement: &str) -> Query {
    Query::Subvert(pattern.to_string(), replacement.to_string())
}
