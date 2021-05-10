use anyhow::{anyhow, Error, Result};
use colored::*;
use isatty::stdout_isatty;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process;
use structopt::StructOpt;

#[derive(Debug)]
enum ColorWhen {
    Always,
    Never,
    Auto,
}

impl std::str::FromStr for ColorWhen {
    type Err = Error;

    fn from_str(s: &str) -> Result<ColorWhen, Error> {
        match s {
            "always" => Ok(ColorWhen::Always),
            "auto" => Ok(ColorWhen::Auto),
            "never" => Ok(ColorWhen::Never),
            _ => Err(anyhow!("Choose between 'always', 'auto', or 'never'")),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "ruplacer",
    after_help = "
EXAMPLES:
    Replace 'foo' with 'bar'
    $ ruplacer foo bar

    Replace 'LastName, FirstName' with 'FirstName LastName'
    $ ruplacer '(\\w+), (\\w+)' '$2 $1'

    Replace '--foo-bar' with '--spam-eggs':
    Note the use of '--' because the pattern and the replacement
    start with two dashes:
    $ ruplacer -- --foo-bar --spam-eggs

    Replace 'FooBar' with 'SpamEggs', 'foo_bar' with 'spam_eggs', ...
    $ ruplacer --subvert FooBar SpamEggs
"
)]
struct Options {
    #[structopt(long = "go", help = "Write the changes to the filesystem")]
    go: bool,

    #[structopt(help = "The pattern to search for")]
    pattern: String,

    #[structopt(help = "The replacement")]
    replacement: String,

    #[structopt(
        parse(from_os_str),
        help = "The source path. Defaults to the working directory"
    )]
    path: Option<PathBuf>,

    #[structopt(
        long = "--no-regex",
        help = "Interpret pattern as a raw string. Default is: regex"
    )]
    no_regex: bool,

    #[structopt(long = "--hidden", help = "Also patch hidden files")]
    hidden: bool,

    #[structopt(long = "--ignored", help = "Also patch ignored files")]
    ignored: bool,

    #[structopt(
        long = "--word-regex",
        short = "-w",
        help = "Interpret pattern as a 'word' regex"
    )]
    word_regex: bool,

    #[structopt(
        long = "--subvert",
        help = "Replace all variants of the pattern (snake_case, CamelCase and so on)"
    )]
    subvert: bool,

    #[structopt(
        short = "t",
        long = "type",
        help = "Only search files matching <file_type> or glob pattern.",
        multiple = true,
        number_of_values = 1
    )]
    selected_file_types: Vec<String>,

    #[structopt(
        short = "T",
        long = "type-not",
        help = "Ignore files matching <file_type> or glob pattern.",
        multiple = true,
        number_of_values = 1
    )]
    ignored_file_types: Vec<String>,

    #[structopt(long = "type-list", help = "List the known file types")]
    file_type_list: bool,

    #[structopt(
        long = "--color",
        help = "Whether to enable colorful output. Choose between 'always', 'auto', or 'never'. Default is 'auto'"
    )]
    color_when: Option<ColorWhen>,
}

fn regex_query_or_die(pattern: &str, replacement: &str, word: bool) -> ruplacer::Query {
    let actual_pattern = if word {
        format!(r"\b({})\b", pattern)
    } else {
        pattern.to_string()
    };
    let re = regex::Regex::new(&actual_pattern);
    if let Err(e) = re {
        eprintln!("{}: {}", "Invalid regex".bold().red(), e);
        process::exit(1);
    }
    let re = re.unwrap();
    ruplacer::query::from_regex(re, replacement)
}

// Set proper env variable so that the colored crate behaves properly.
// See: https://bixense.com/clicolors/
fn configure_color(when: &ColorWhen) {
    match when {
        ColorWhen::Always => std::env::set_var("CLICOLOR_FORCE", "1"),
        ColorWhen::Never => std::env::set_var("CLICOLOR", "0"),
        ColorWhen::Auto => {
            if stdout_isatty() {
                std::env::set_var("CLICOLOR", "1")
            } else {
                std::env::set_var("CLICOLOR", "0")
            }
        }
    }
}

fn print_stats(stats: &ruplacer::Stats, dry_run: bool) {
    if dry_run {
        print!("Would perform ")
    } else {
        print!("Performed ")
    }
    println!("{}", stats)
}

fn on_type_list() {
    println!("Known file types:");
    let mut types_builder = ignore::types::TypesBuilder::new();
    types_builder.add_defaults();
    for def in types_builder.definitions() {
        let name = def.name();
        let globs = def.globs();
        println!("{}: {}", name.bold(), globs.join(", "));
    }
}

fn main() -> Result<()> {
    let opt = Options::from_args();
    let Options {
        color_when,
        file_type_list,
        go,
        hidden,
        ignored,
        ignored_file_types,
        no_regex,
        path,
        pattern,
        replacement,
        selected_file_types,
        subvert,
        word_regex,
    } = opt;

    if file_type_list {
        on_type_list();
        return Ok(());
    }

    let dry_run = !go;

    let color_when = &color_when.unwrap_or(ColorWhen::Auto);
    configure_color(&color_when);

    let query = if no_regex {
        ruplacer::query::substring(&pattern, &replacement)
    } else if subvert {
        ruplacer::query::subvert(&pattern, &replacement)
    } else {
        regex_query_or_die(&pattern, &replacement, word_regex)
    };

    let settings = ruplacer::Settings {
        dry_run,
        hidden,
        ignored,
        selected_file_types,
        ignored_file_types,
    };

    let path = path.unwrap_or_else(|| Path::new(".").to_path_buf());
    if path.to_string_lossy() == "-" {
        run_on_stdin(query)
    } else {
        run_on_directory(path, settings, query)
    }
}

fn run_on_stdin(query: ruplacer::Query) -> Result<()> {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        let replacement = ruplacer::replace(&line, &query);
        if let Some(replacement) = replacement {
            println!("{}", replacement.output());
        } else {
            println!("{}", line);
        }
    }
    Ok(())
}

fn run_on_directory(
    path: PathBuf,
    settings: ruplacer::Settings,
    query: ruplacer::Query,
) -> Result<()> {
    let dry_run = settings.dry_run;
    let mut directory_patcher = ruplacer::DirectoryPatcher::new(path, settings);
    directory_patcher.run(&query)?;
    let stats = directory_patcher.stats();
    if stats.num_replacements == 0 {
        #[allow(clippy::print_literal)]
        {
            eprintln!("{}: {}", "Error".bold().red(), "nothing found to replace");
        }
        process::exit(2);
    }
    print_stats(&stats, dry_run);
    if dry_run {
        println!("Re-run ruplacer with --go to write these changes to the filesystem");
    }
    Ok(())
}
