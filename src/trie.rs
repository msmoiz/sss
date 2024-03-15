use std::collections::HashMap;

struct Trie {
    next: HashMap<char, Trie>,
    occs: Vec<usize>,
}

impl Trie {
    fn new(corpus: &[&'static str]) -> Self {
        let mut root = Self::node();

        for (i, line) in corpus.iter().enumerate() {
            line.split_ascii_whitespace().for_each(|word| {
                let mut current = &mut root;
                for char in word.chars() {
                    if current.next.contains_key(&char) {
                        current = current.next.get_mut(&char).unwrap();
                    } else {
                        current.next.insert(char, Self::node());
                        current = current.next.get_mut(&char).unwrap();
                    }
                }
                current.occs.push(i);
            })
        }

        root
    }

    fn node() -> Self {
        Self {
            next: HashMap::new(),
            occs: Vec::new(),
        }
    }

    fn find(&self, word: &str) -> Option<Vec<usize>> {
        let mut current = self;
        for char in word.chars() {
            match current.next.get(&char) {
                Some(node) => current = node,
                None => return None,
            }
        }
        Some(current.occs.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::Trie;

    const CORPUS: [&'static str; 10] = [
        "Cats nap often, basking in warm spots.",
        "Raindrops patter softly on windowpanes.",
        "Stars twinkle brightly in the night.",
        "Rivers flow quietly through lush valleys.",
        "Birds chirp merrily at dawn's break.",
        "Autumn leaves rustle underfoot, falling gently.",
        "Waves crash rhythmically against rocky shores.",
        "Children giggle while playing in parks.",
        "Sunflowers turn eagerly towards the sun.",
        "Snowflakes drift down gracefully from the sky.",
    ];

    #[test]
    fn test() {
        let index = Trie::new(&CORPUS);

        let in_occ = index.find("in");
        assert_eq!(in_occ, Some(vec![0, 2, 7]));

        let in_occ = index.find("on");
        assert_eq!(in_occ, Some(vec![1]));

        let in_occ = index.find("the");
        assert_eq!(in_occ, Some(vec![2, 8, 9]));
    }
}
