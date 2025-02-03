use std::fs;
use std::iter::Peekable;

pub struct EntityParser(Vec<(String, Vec<u32>)>);

impl EntityParser {
    fn parse_unicode(unicode: u32) -> Option<char> {
        char::from_u32(unicode)
    }

    pub fn new() -> Self {
        let Ok(source) = fs::read_to_string("./entities.in") else {
            panic!("Cannot open entities.in source");
        };

        let source = source
            .lines()
            .map(|line| {
                let mut spec = line.split_whitespace();

                let Some(name) = spec.next() else {
                    panic!("Missing entity name")
                };

                let codepoints = spec
                    .filter_map(|code| code.parse::<u32>().ok())
                    .collect::<Vec<u32>>();

                (name.to_owned(), codepoints)
            })
            .collect::<_>();

        EntityParser(source)
    }

    // TODO: Optimize to avoid vector allocation
    // and search for the codepoints linearly
    pub fn consume<I>(&self, input: &mut Peekable<I>) -> Option<String>
    where
        I: Iterator<Item = char>,
    {
        let mut acc = "".to_string();

        let mut candidates = self
            .0
            .iter()
            .filter(|(name, _)| name.starts_with(&acc))
            .collect::<Vec<&(String, Vec<u32>)>>();

        while let Some(next) = input.peek() {
            let mut local = acc.clone();
            local.push(*next);

            let next_candidates = candidates
                .iter()
                .copied()
                .filter(|(name, _)| name.starts_with(&local))
                .collect::<Vec<&(String, Vec<u32>)>>();

            if next_candidates.is_empty() {
                break;
            }

            candidates = next_candidates;

            input.next(); // consume
            acc = local;
        }

        if candidates.iter().any(|(name, _)| name == &acc) {
            return self.execute(&acc);
        }

        None
    }

    fn execute(&self, input: &str) -> Option<String> {
        if let Some((_, codepoints)) = self.0.iter().find(|(name, _)| name == input) {
            return Some(
                codepoints
                    .iter()
                    .filter_map(|&code| Self::parse_unicode(code))
                    .collect::<String>(),
            );
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_angled_brackets() {
        let parser = EntityParser::new();

        // let src = "&lt;div&gt;".to_string();
        let input = "&lt;".chars();
        let mut input_peek = input.peekable();

        assert_eq!(parser.consume(&mut input_peek), Some("<".to_string()));

        let input = "&lt".chars();
        let mut input_peek = input.peekable();

        assert_eq!(parser.consume(&mut input_peek), Some("<".to_string()));
    }
}
