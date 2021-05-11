use inflector::string::pluralize::to_plural;

#[derive(Default, Debug)]
pub struct Stats {
    pub matching_files: usize,
    pub num_replacements: usize,
}

impl Stats {
    pub fn update(&mut self, num_replacements: usize) {
        self.matching_files += 1;
        self.num_replacements += num_replacements;
    }
}

fn pluralize(input: &str, num: usize) -> String {
    if num > 1 {
        to_plural(input)
    } else {
        input.to_string()
    }
}

impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let file_string = pluralize("file", self.matching_files);
        let replacements_string = pluralize("replacement", self.num_replacements);
        write!(
            f,
            "{} {} on {} matching {}",
            self.num_replacements, replacements_string, self.matching_files, file_string
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_to_string() {
        let stats = Stats {
            matching_files: 2,
            num_replacements: 4,
        };
        let actual = stats.to_string();
        assert_eq!(actual, "4 replacements on 2 matching files");

        let stats = Stats {
            matching_files: 1,
            num_replacements: 2,
        };
        let actual = stats.to_string();
        assert_eq!(actual, "2 replacements on 1 matching file");
    }
}
