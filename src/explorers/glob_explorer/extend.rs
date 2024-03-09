use std::collections::HashSet;

/// A data type used to help parse extended glob patterns into basic glob patterns.
#[derive(Debug)]
enum ExtendGlobToken {
    Literal(char),
    Subpatterns(Vec<String>),
}

/// A helper function which takes an extended glob pattern and returns an equivalent set of basic glob patterns.
pub fn extend_glob_pattern(pattern: &str) -> HashSet<String> {
    // tokens will be either a literal character, or a subpattern at depth >= 1
    // note: subpatterns deeper than depth 1 will be parsed recursively
    let mut tokens: Vec<ExtendGlobToken> = Vec::new();

    /// A character which exists on or beyond depth = 1. This will be parsed later in a recursive step
    fn push_subpattern_character(tokens: &mut Vec<ExtendGlobToken>, c: char) {
        match tokens.last_mut().unwrap() {
            ExtendGlobToken::Subpatterns(subpatterns) => {
                subpatterns.last_mut().unwrap().push(c);
            }
            _ => panic!("Invalid state"),
        }
    }

    let mut depth = 0;
    let mut escaped = false;
    for c in pattern.chars() {
        if escaped {
            escaped = false;

            if depth == 0 {
                tokens.push(ExtendGlobToken::Literal(c));
            } else {
                push_subpattern_character(&mut tokens, c);
            }

            continue;
        }

        match c {
            '\\' => {
                escaped = true;

                if depth == 0 {
                    tokens.push(ExtendGlobToken::Literal(c));
                } else {
                    push_subpattern_character(&mut tokens, c);
                }
            }
            '{' => {
                depth += 1;

                if depth == 1 {
                    // prepare for subpatterns at depth 1
                    tokens.push(ExtendGlobToken::Subpatterns(vec!["".to_owned()]));
                } else {
                    push_subpattern_character(&mut tokens, c);
                }
            }
            '}' => {
                depth -= 1;

                if depth == 0 {
                    // closing the subpattern at depth 1: extend subpatterns recursively
                    let mut extended_basic_glob_patterns: Vec<String> = Vec::new();

                    match tokens.pop().unwrap() {
                        ExtendGlobToken::Subpatterns(subpatterns) => {
                            for subpattern in subpatterns {
                                extended_basic_glob_patterns
                                    .extend(extend_glob_pattern(&subpattern));
                            }
                        }
                        ExtendGlobToken::Literal(_) => panic!("Invalid state"),
                    }

                    tokens.push(ExtendGlobToken::Subpatterns(extended_basic_glob_patterns));
                } else {
                    push_subpattern_character(&mut tokens, c);
                }
            }
            ',' => {
                if depth == 0 {
                    tokens.push(ExtendGlobToken::Literal(c));
                } else if depth == 1 {
                    // delimits two subpatterns in the depth 1 disjuction; prepare for the next subpattern
                    match tokens.last_mut().unwrap() {
                        ExtendGlobToken::Subpatterns(subpatterns) => {
                            subpatterns.push("".to_owned());
                        }
                        _ => panic!("Invalid state"),
                    }
                } else {
                    push_subpattern_character(&mut tokens, c);
                }
            }
            _ => {
                if depth == 0 {
                    tokens.push(ExtendGlobToken::Literal(c));
                } else {
                    push_subpattern_character(&mut tokens, c);
                }
            }
        }
    }

    // reconstruct the basic glob patterns from the tokens; this is basically the cartesian product
    let mut basic_glob_patterns: Vec<String> = vec!["".to_owned()];
    for token in tokens {
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
        let actual: std::collections::HashSet<String> =
            extend_glob_pattern(pattern).into_iter().collect();
        let expected: std::collections::HashSet<String> =
            expected.iter().map(|s| s.to_string()).collect();
        assert_eq!(actual, expected);
    }
}
