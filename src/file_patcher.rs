use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::query::Query;
use crate::replace;

pub struct FilePatcher {
    path: PathBuf,
    new_contents: String,
    num_replacements: usize,
    num_lines: usize,
}

impl FilePatcher {
    pub fn new(path: &Path, query: &Query) -> Result<Option<FilePatcher>> {
        let mut num_replacements = 0;
        let mut num_lines = 0;
        let file =
            File::open(&path).with_context(|| format!("Could not open {}", path.display()))?;
        let reader = BufReader::new(file);
        let mut new_contents = String::new();
        // Note: not using lines() because we need to preserve the line endings
        // when writing the file later on
        for (num, chunk) in reader.split(b'\n').enumerate() {
            let chunk = chunk.with_context(|| format!("Error while reading {}", path.display()))?;
            let line = std::str::from_utf8(&chunk);
            if line.is_err() {
                return Ok(None);
            }
            let line = line.unwrap();
            let replacement = replace(&line, &query);
            match replacement {
                None => new_contents.push_str(&line),
                Some(replacement) => {
                    num_lines += 1;
                    num_replacements += replacement.num_fragments();
                    let lineno = num + 1;
                    let prefix = format!("{}:{} ", path.display(), lineno);
                    let new_line = replacement.output();
                    replacement.print_self(&prefix);
                    new_contents.push_str(&new_line);
                }
            }
            new_contents.push('\n');
        }
        Ok(Some(FilePatcher {
            path: path.to_path_buf(),
            new_contents,
            num_lines,
            num_replacements,
        }))
    }

    pub(crate) fn num_replacements(&self) -> usize {
        self.num_replacements
    }

    pub(crate) fn num_lines(&self) -> usize {
        self.num_lines
    }

    pub fn run(&self) -> Result<()> {
        std::fs::write(&self.path, &self.new_contents)
            .with_context(|| format!("Could not write {}", self.path.display()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Query;
    use std::fs;

    #[test]
    fn test_patch_file() {
        let temp_dir = tempdir::TempDir::new("test-ruplacer").unwrap();
        let file_path = temp_dir.path().join("foo.txt");
        fs::write(&file_path, "first line\nI say: old is nice\nlast line\n").unwrap();
        let query = Query::substring("old", "new");
        let file_patcher = FilePatcher::new(&file_path, &query).unwrap();
        file_patcher.unwrap().run().unwrap();
        let actual = fs::read_to_string(&file_path).unwrap();
        let expected = "first line\nI say: new is nice\nlast line\n";
        assert_eq!(actual, expected);
    }
}
