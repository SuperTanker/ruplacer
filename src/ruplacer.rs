#![allow(dead_code)]
use crate::query::Query;
use colored::*;
use regex::Regex;
use std::path::Path;

#[derive(Debug)]
pub struct Replacement<'a> {
    fragments: Fragments,
    input: &'a str,
    output: String,
}

impl<'a> Replacement<'a> {
    pub fn output(&self) -> &str {
        &self.output
    }

    pub fn print_self(&self, path: &Path, lineno: usize) {
        let prefix = format!(
            "{}:{}",
            path.display().to_string().bold(),
            lineno.to_string()
        );
        print!("{} {}", prefix, "--- ".red());
        let mut current_index = 0;
        for input_fragment in &self.fragments.inputs {
            let Fragment {
                index: input_index,
                text: input_text,
            } = input_fragment;
            print!("{}", &self.input[current_index..*input_index]);
            print!("{}", input_text.red().underline());
            current_index = input_index + input_text.len();
        }
        println!("{}", &self.input[current_index..]);

        print!("{} {}", prefix, "+++ ".green());
        let mut current_index = 0;
        for output_fragment in &self.fragments.outputs {
            let Fragment {
                index: output_index,
                text: output_text,
            } = output_fragment;
            print!("{}", &self.output[current_index..*output_index]);
            print!("{}", output_text.green().underline());
            current_index = output_index + output_text.len();
        }
        println!("{}", &self.output[current_index..]);
    }
}

pub fn replace<'a>(input: &'a str, query: &Query) -> Option<Replacement<'a>> {
    let fragments = get_fragments(input, query);
    if fragments.is_empty() {
        return None;
    }
    let output = get_output(input, &fragments);
    Some(Replacement {
        fragments,
        input,
        output,
    })
}

trait Replacer {
    // return position of the match, input_text, output_text, or None
    fn find_and_replace(&self, buff: &str) -> Option<(usize, String, String)>;
}

struct SubstringReplacer<'a> {
    pattern: &'a str,
    replacement: &'a str,
}

impl<'a> SubstringReplacer<'a> {
    fn new(pattern: &'a str, replacement: &'a str) -> Self {
        Self {
            pattern,
            replacement,
        }
    }
}

impl<'a> Replacer for SubstringReplacer<'a> {
    fn find_and_replace(&self, buff: &str) -> Option<(usize, String, String)> {
        let pos = buff.find(&self.pattern)?;
        Some((pos, self.pattern.to_string(), self.replacement.to_string()))
    }
}

struct SubvertReplacer<'a> {
    patterns: &'a [String],
    replacements: &'a [String],
}

impl<'a> SubvertReplacer<'a> {
    fn new(patterns: &'a [String], replacements: &'a [String]) -> Self {
        Self {
            patterns,
            replacements,
        }
    }
}

impl<'a> Replacer for SubvertReplacer<'a> {
    fn find_and_replace(&self, buff: &str) -> Option<(usize, String, String)> {
        // We need to return the best possible match, other wise we may
        // replace FooBar with SpamEggs *before* replacing foo-bar with spam-eggs
        let mut best_pos = buff.len();
        let mut best_index = None;
        for (i, pattern) in self.patterns.iter().enumerate() {
            let pos = buff.find(pattern);
            match pos {
                Some(p) if p < best_pos => {
                    best_pos = p;
                    best_index = Some(i)
                }
                // Not found, or in a later position than the best, ignore
                _ => {}
            }
        }

        let best_index = best_index?;
        Some((
            best_pos,
            self.patterns[best_index].to_string(),
            self.replacements[best_index].to_string(),
        ))
    }
}

struct RegexReplacer<'a> {
    regex: &'a Regex,
    replacement: &'a str,
}

impl<'a> RegexReplacer<'a> {
    fn new(regex: &'a Regex, replacement: &'a str) -> Self {
        Self { regex, replacement }
    }
}

impl<'a> Replacer for RegexReplacer<'a> {
    fn find_and_replace(&self, buff: &str) -> Option<(usize, String, String)> {
        let regex_match = self.regex.find(buff)?;
        let pos = regex_match.start();
        let input_text = regex_match.as_str();
        let output_text = self.regex.replacen(input_text, 1, self.replacement);
        Some((pos, input_text.to_string(), output_text.to_string()))
    }
}

/// Represent a fragment of text, similar to the data structure returned
/// by String::match_indices
#[derive(Debug)]
struct Fragment {
    index: usize,
    text: String,
}

/// inputs: framgents that are found
/// outputs: replacements for the fragments that were found
#[derive(Debug)]
struct Fragments {
    inputs: Vec<Fragment>,
    outputs: Vec<Fragment>,
}

impl Fragments {
    fn new() -> Self {
        Self {
            inputs: vec![],
            outputs: vec![],
        }
    }

    fn is_empty(&self) -> bool {
        self.inputs.is_empty()
    }

    fn add(&mut self, input: (usize, &str), output: (usize, &str)) {
        // invariant: self.inputs and self.outputs should always have the same lenght
        self.inputs.push(Fragment {
            index: input.0,
            text: input.1.to_string(),
        });
        self.outputs.push(Fragment {
            index: output.0,
            text: output.1.to_string(),
        });
    }
}

/// Return a list of fragments for input string and ouptut string
/// Both lists of framgents will be used for:
///    - computing the output string
///    - printing the patch
fn get_fragments(input: &str, query: &Query) -> Fragments {
    match query {
        Query::Substring(pattern, replacement) => {
            let finder = SubstringReplacer::new(&pattern, &replacement);
            get_fragments_with_finder(input, finder)
        }
        Query::Regex(regex, replacement) => {
            let finder = RegexReplacer::new(&regex, &replacement);
            get_fragments_with_finder(input, finder)
        }
        Query::Subvert(patterns, replacements) => {
            let finder = SubvertReplacer::new(&patterns, &replacements);
            get_fragments_with_finder(input, finder)
        }
    }
}

fn get_output(input: &str, Fragments { inputs, outputs }: &Fragments) -> String {
    let mut current_index = 0;
    let mut output = String::new();
    for (input_fragment, output_fragment) in inputs.iter().zip(outputs.iter()) {
        let Fragment {
            text: input_text,
            index: input_index,
        } = input_fragment;

        let Fragment {
            text: output_text, ..
        } = output_fragment;

        output.push_str(&input[current_index..*input_index]);
        output.push_str(&output_text);
        current_index = input_index + input_text.len();
    }
    output.push_str(&input[current_index..]);
    output
}

fn get_fragments_with_finder(input: &str, finder: impl Replacer) -> Fragments {
    let mut fragments = Fragments::new();
    let mut input_index = 0;
    let mut output_index = 0;
    let mut buff = input;
    while let Some(res) = finder.find_and_replace(buff) {
        let (pos, input_text, output_text) = res;
        input_index += pos;
        output_index += pos;
        fragments.add((input_index, &input_text), (output_index, &output_text));
        let new_start = input_index + input_text.len();
        buff = &input[new_start..];
        input_index += input_text.len();
        output_index += output_text.len();
    }

    fragments
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::query;
    use regex::Regex;

    #[test]
    fn test_substring_1() {
        let input = "Mon thé c'est le meilleur des thés !";
        let pattern = "thé";
        let replacement = "café";
        let query = query::substring(pattern, replacement);
        let replacement = replace(input, &query).unwrap();
        assert_eq!(
            replacement.output(),
            "Mon café c'est le meilleur des cafés !"
        );
    }

    #[test]
    fn test_substring_2() {
        let input = "old old old";
        let pattern = "old";
        let replacement = "new";
        let query = query::substring(pattern, replacement);
        let replacement = replace(input, &query).unwrap();
        assert_eq!(replacement.output(), "new new new");
    }

    #[test]
    fn test_patch() {
        let input = "Top: old is nice";
        let pattern = "old";
        let replacement = "new";
        let query = query::substring(pattern, replacement);
        let replacement = replace(input, &query).unwrap();
        let path = Path::new("toto");
        replacement.print_self(&path, 3);
    }

    #[test]
    fn test_subvert() {
        let input = "let foo_bar = FooBar::new();";
        let pattern = "foo_bar";
        let replacement = "spam_eggs";
        let query = query::subvert(pattern, replacement);
        let replacement = replace(input, &query).unwrap();
        assert_eq!(replacement.output(), "let spam_eggs = SpamEggs::new();");
    }

    #[test]
    fn test_regex_with_substitutions() {
        let input = "first, second";
        let regex = Regex::new(r"(\w+), (\w+)").unwrap();
        let query = query::from_regex(regex, r"$2 $1");
        let replacement = replace(input, &query).unwrap();
        assert_eq!(replacement.output(), "second first");
    }

    #[test]
    fn test_simple_regex() {
        let input = "old is old";
        let regex = Regex::new("old").unwrap();
        let query = query::from_regex(regex, "new");
        let replacement = replace(input, &query).unwrap();
        assert_eq!(replacement.output(), "new is new");
    }
}
