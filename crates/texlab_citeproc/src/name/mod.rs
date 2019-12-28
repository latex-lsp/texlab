// Ported from: https://github.com/michel-kraemer/citeproc-java/blob/master/citeproc-java/grammars/InternalName.g4
// Michel Kraemer
// Apache License 2.0
mod parser {
    #![allow(warnings)]
    include!(concat!(env!("OUT_DIR"), "/name/parser.rs"));
}

use self::parser::NamesParser;
use citeproc_io::Name;

pub fn parse(input: &str) -> Vec<Name> {
    let parser = NamesParser::new();
    parser.parse(input).unwrap_or_else(|_| {
        vec![Name::Literal {
            literal: input.into(),
        }]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use citeproc_io::PersonName;

    #[test]
    fn test_family_only() {
        let name = Name::Person(PersonName {
            family: Some("Thompson".into()),
            given: None,
            non_dropping_particle: None,
            dropping_particle: None,
            suffix: None,
        });
        assert_eq!(parse("Thompson"), vec![name]);
    }

    #[test]
    fn test_simple() {
        let name = Name::Person(PersonName {
            family: Some("Thompson".into()),
            given: Some("Ken".into()),
            non_dropping_particle: None,
            dropping_particle: None,
            suffix: None,
        });
        assert_eq!(parse("Ken Thompson"), vec![name]);
    }

    #[test]
    fn test_middle_name() {
        let name = Name::Person(PersonName {
            family: Some("Ritchie".into()),
            given: Some("Dennis M.".into()),
            non_dropping_particle: None,
            dropping_particle: None,
            suffix: None,
        });
        assert_eq!(parse("Dennis M. Ritchie"), vec![name]);
    }

    #[test]
    fn test_initials() {
        let name = Name::Person(PersonName {
            family: Some("Johnson".into()),
            given: Some("S. C.".into()),
            non_dropping_particle: None,
            dropping_particle: None,
            suffix: None,
        });
        assert_eq!(parse("S. C. Johnson"), vec![name]);
    }

    #[test]
    fn test_non_dropping_particle() {
        let name = Name::Person(PersonName {
            family: Some("Gerwen".into()),
            given: Some("Michael".into()),
            non_dropping_particle: Some("van".into()),
            dropping_particle: None,
            suffix: None,
        });
        assert_eq!(parse("Michael van Gerwen"), vec![name]);
    }

    #[test]
    fn test_non_dropping_particle_family_only() {
        let name = Name::Person(PersonName {
            family: Some("Gerwen".into()),
            given: None,
            non_dropping_particle: Some("van".into()),
            dropping_particle: None,
            suffix: None,
        });
        assert_eq!(parse("van Gerwen"), vec![name]);
    }

    #[test]
    fn test_comma() {
        let name = Name::Person(PersonName {
            family: Some("Thompson".into()),
            given: Some("Ken".into()),
            non_dropping_particle: None,
            dropping_particle: None,
            suffix: None,
        });
        assert_eq!(parse("Thompson, Ken"), vec![name]);
    }

    #[test]
    fn test_comma_junior() {
        let name = Name::Person(PersonName {
            family: Some("Friedman".into()),
            given: Some("George".into()),
            non_dropping_particle: None,
            dropping_particle: None,
            suffix: Some("Jr.".into()),
        });
        assert_eq!(parse("Friedman, Jr., George"), vec![name]);
    }

    #[test]
    fn test_comma_no_junior() {
        let name = Name::Person(PersonName {
            family: Some("Familya Familyb".into()),
            given: Some("Given".into()),
            non_dropping_particle: None,
            dropping_particle: None,
            suffix: None,
        });
        assert_eq!(parse("Familya Familyb, Given"), vec![name]);
    }

    #[test]
    fn test_comma_initials() {
        let name = Name::Person(PersonName {
            family: Some("Ritchie".into()),
            given: Some("Dennis M.".into()),
            non_dropping_particle: None,
            dropping_particle: None,
            suffix: None,
        });
        assert_eq!(parse("Ritchie, Dennis M."), vec![name]);
    }

    #[test]
    fn test_comma_non_dropping_particle() {
        let name = Name::Person(PersonName {
            family: Some("Gerwen".into()),
            given: Some("Michael".into()),
            non_dropping_particle: Some("van".into()),
            dropping_particle: None,
            suffix: None,
        });
        assert_eq!(parse("van Gerwen, Michael"), vec![name]);
    }

    #[test]
    fn test_comma_non_dropping_particles() {
        let name = Name::Person(PersonName {
            family: Some("Voort".into()),
            given: Some("Vincent".into()),
            non_dropping_particle: Some("Van der".into()),
            dropping_particle: None,
            suffix: None,
        });
        assert_eq!(parse("Van der Voort, Vincent"), vec![name]);
    }

    #[test]
    fn test_and() {
        let name1 = Name::Person(PersonName {
            family: Some("Gerwen".into()),
            given: Some("Michael".into()),
            non_dropping_particle: Some("van".into()),
            dropping_particle: None,
            suffix: None,
        });
        let name2 = Name::Person(PersonName {
            family: Some("Voort".into()),
            given: Some("Vincent".into()),
            non_dropping_particle: Some("van der".into()),
            dropping_particle: None,
            suffix: None,
        });
        assert_eq!(
            parse("Michael van Gerwen and Vincent van der Voort"),
            vec![name1, name2]
        );
    }

    #[test]
    fn test_and_comma1() {
        let name1 = Name::Person(PersonName {
            family: Some("Gerwen".into()),
            given: Some("Michael".into()),
            non_dropping_particle: Some("van".into()),
            dropping_particle: None,
            suffix: None,
        });
        let name2 = Name::Person(PersonName {
            family: Some("Voort".into()),
            given: Some("Vincent".into()),
            non_dropping_particle: Some("Van der".into()),
            dropping_particle: None,
            suffix: None,
        });
        assert_eq!(
            parse("van Gerwen, Michael and Van der Voort, Vincent"),
            vec![name1, name2]
        );
    }

    #[test]
    fn test_and_comma2() {
        let name1 = Name::Person(PersonName {
            family: Some("Gerwen".into()),
            given: Some("Michael".into()),
            non_dropping_particle: Some("van".into()),
            dropping_particle: None,
            suffix: None,
        });
        let name2 = Name::Person(PersonName {
            family: Some("Voort".into()),
            given: Some("Vincent".into()),
            non_dropping_particle: Some("van der".into()),
            dropping_particle: None,
            suffix: None,
        });
        assert_eq!(
            parse("van Gerwen, Michael and van der Voort, Vincent"),
            vec![name1, name2]
        );
    }

    #[test]
    fn test_and_comma_mix() {
        let name1 = Name::Person(PersonName {
            family: Some("Gerwen".into()),
            given: Some("Michael".into()),
            non_dropping_particle: Some("van".into()),
            dropping_particle: None,
            suffix: None,
        });
        let name2 = Name::Person(PersonName {
            family: Some("Voort".into()),
            given: Some("Vincent".into()),
            non_dropping_particle: Some("van der".into()),
            dropping_particle: None,
            suffix: None,
        });
        assert_eq!(
            parse("van Gerwen, Michael and Vincent van der Voort"),
            vec![name1, name2]
        );
    }

    #[test]
    fn test_junior() {
        let name = Name::Person(PersonName {
            family: Some("Friedman".into()),
            given: Some("George".into()),
            non_dropping_particle: None,
            dropping_particle: None,
            suffix: Some("Jr.".into()),
        });
        assert_eq!(parse("George Friedman, Jr."), vec![name]);
    }

    #[test]
    fn test_non_parseable() {
        let literal = "Jerry Peek and Tim O'Reilly and Mike Loukides and other authors of the Nutshell handbooks";
        let name = Name::Literal {
            literal: literal.into(),
        };
        assert_eq!(parse(literal), vec![name]);
    }
}
