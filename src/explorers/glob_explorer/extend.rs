use std::collections::HashSet;

/// A data type used to help parse extended glob patterns into basic glob patterns.
#[derive(Debug)]
enum ExtendGlobToken {
    /// A simple top-level character. E.g., 'a'
    Literal(char),

    /// A glob subpattern. E.g., '{a,b,{00,11}!}' as ["a", "b", "{00,11}!"].
    /// These patterns are parsed later in another recursive step
    Subpatterns(Vec<String>),
}

/// A builder-flavoured struct that helps convert extended glob patterns into a collection of basic glob patterns.
pub struct ExtendedGlobPatternBuilder {
    /// the individual components of the glob pattern
    tokens: Vec<ExtendGlobToken>,

    /// the current parsing depth (tracks nested {}) - recall; we only parse depth 1 subpatterns in this step, and
    /// rely on recursive steps to parse nested subpatterns
    depth: usize,

    /// flags whether the previous character was a backslash (\) or not
    escaped: bool,
}

impl ExtendedGlobPatternBuilder {
    /// A helper function that converts an extended glob pattern into a collection of basic glob patterns.
    pub fn from_pattern(pattern: &str) -> Self {
        let mut builder = Self::new();
        for c in pattern.chars() {
            builder.character(c);
        }
        return builder;
    }

    /// Construct a new empty extended glob pattern builder. Helpful when calling `::character` directly, but
    /// it's generally more friendly to use `::from_pattern` instead.
    pub fn new() -> Self {
        return Self {
            tokens: Vec::new(),
            depth: 0,
            escaped: false,
        };
    }

    /// Parse a single additional character from the (potentially) extended glob pattern.
    pub fn character(&mut self, c: char) {
        if self.escaped {
            self.escaped = false;
            self.normal_character(c);
            return;
        }

        match c {
            '{' => self.open_parenthesis(),
            '}' => self.close_parenthesis(),
            ',' => self.comma(),
            '\\' => {
                self.escaped = true;
                self.normal_character(c);
            }
            _ => self.normal_character(c),
        }
    }

    /// Converts the tokenized extended glob pattern into a collection of basic glob patterns.
    pub fn build(self) -> HashSet<String> {
        let mut basic_glob_patterns: Vec<String> = vec!["".to_owned()];
        for token in self.tokens {
            match token {
                ExtendGlobToken::Literal(c) => {
                    for pattern in basic_glob_patterns.iter_mut() {
                        pattern.push(c);
                    }
                }
                ExtendGlobToken::Subpatterns(subpatterns) => {
                    let mut new_basic_glob_patterns: Vec<String> =
                        Vec::with_capacity(basic_glob_patterns.len() * subpatterns.len());

                    for pattern in basic_glob_patterns.iter_mut() {
                        for subpattern in &subpatterns {
                            new_basic_glob_patterns.push(format!("{pattern}{subpattern}"));
                        }
                    }

                    basic_glob_patterns = new_basic_glob_patterns;
                }
            }
        }

        return basic_glob_patterns.into_iter().collect();
    }

    fn comma(&mut self) {
        if self.depth == 0 {
            self.tokens.push(ExtendGlobToken::Literal(','));
        } else if self.depth == 1 {
            // delimits two subpatterns in the depth 1 disjuction; prepare for the next subpattern
            match self.tokens.last_mut().unwrap() {
                ExtendGlobToken::Subpatterns(subpatterns) => {
                    subpatterns.push("".to_owned());
                }
                _ => panic!("Comma was expected to delimit two subpatterns at depth 1. Last token is the wrong type"),
            }
        } else {
            self.push_subpattern_character(',');
        }
    }

    fn normal_character(&mut self, c: char) {
        if self.depth == 0 {
            self.tokens.push(ExtendGlobToken::Literal(c));
        } else {
            self.push_subpattern_character(c);
        }
    }

    fn open_parenthesis(&mut self) {
        self.depth += 1;

        if self.depth == 1 {
            // prepare for subpatterns at depth 1
            self.tokens
                .push(ExtendGlobToken::Subpatterns(vec!["".to_owned()]));
        } else {
            self.push_subpattern_character('{');
        }
    }

    fn close_parenthesis(&mut self) {
        self.depth -= 1;

        if self.depth == 0 {
            // closing the subpattern at depth 1: extend subpatterns recursively
            let mut extended_basic_glob_patterns: Vec<String> = Vec::new();

            match self.tokens.pop().unwrap() {
                ExtendGlobToken::Subpatterns(subpatterns) => {
                    for subpattern in subpatterns {
                        extended_basic_glob_patterns
                            .extend(ExtendedGlobPatternBuilder::from_pattern(&subpattern).build());
                    }
                }
                _ => panic!("Cannot close subpattern when last token is not a subpattern"),
            }

            self.tokens
                .push(ExtendGlobToken::Subpatterns(extended_basic_glob_patterns));
        } else {
            self.push_subpattern_character('}');
        }
    }

    fn push_subpattern_character(&mut self, c: char) {
        match self.tokens.last_mut().unwrap() {
            ExtendGlobToken::Subpatterns(subpatterns) => {
                subpatterns.last_mut().unwrap().push(c);
            }
            _ => panic!("Cannot push subpattern character when last token is not a subpattern"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case("base case", vec!["base case"])]
    #[case("escaped \\{ is OK", vec!["escaped \\{ is OK"])]
    #[case("commas, are OK", vec!["commas, are OK"])]
    #[case("{no reason for expansion}", vec!["no reason for expansion"])]
    #[case("{no reason for \\{\\} expansion}", vec!["no reason for \\{\\} expansion"])]
    #[case("{a,b}", vec!["a", "b"])]
    #[case("{apple,banana}", vec!["apple", "banana"])]
    #[case("{apple,banana,carrot}", vec!["apple", "banana", "carrot"])]
    #[case("config.{yml,yaml}", vec!["config.yml", "config.yaml"])]
    #[case("{apple,pumpkin,strawberry} pie", vec!["apple pie", "pumpkin pie", "strawberry pie"])]
    #[case("{a,b,c}{1,2}", vec!["a1", "a2", "b1", "b2", "c1", "c2"])]
    #[case("{a,b}{1,2}{!,?}", vec!["a1!", "a2!", "b1!", "b2!", "a1?", "a2?", "b1?", "b2?"])]
    #[case("a{b,{c,d}}", vec!["ab", "ac", "ad"])]
    #[case("{aa{bb,cc,dd{e,f}},why even}.", vec!["why even.", "aabb.", "aacc.", "aadde.", "aaddf."])]
    fn given_extended_glob_pattern_when_extend_glob_pattern_then_converts_into_multiple_basic_patterns(
        #[case] pattern: &str,
        #[case] expected: Vec<&str>,
    ) {
        println!("Glob pattern: {pattern}");
        let actual: std::collections::HashSet<String> =
            ExtendedGlobPatternBuilder::from_pattern(pattern)
                .build()
                .into_iter()
                .collect();
        let expected: std::collections::HashSet<String> =
            expected.iter().map(|s| s.to_string()).collect();
        assert_eq!(actual, expected);
    }
}
