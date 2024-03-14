use std::collections::HashMap;

struct Index {
    inner: HashMap<&'static str, Vec<usize>>,
}

impl Index {
    fn new(corpus: &[&'static str]) -> Self {
        let mut inner: HashMap<&'static str, Vec<usize>> = HashMap::new();

        for (i, line) in corpus.iter().enumerate() {
            line.split_ascii_whitespace()
                .for_each(|word| match inner.get_mut(word) {
                    Some(occurrences) => occurrences.push(i),
                    None => {
                        inner.insert(word, vec![i]);
                    }
                })
        }

        Self { inner }
    }

    fn find(&self, word: &str) -> Option<Vec<usize>> {
        match self.inner.get(word) {
            Some(occurrences) => Some(occurrences.clone()),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Index;

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
        let index = Index::new(&CORPUS);

        let in_occ = index.find("in");
        assert_eq!(in_occ, Some(vec![0, 2, 7]));

        let in_occ = index.find("on");
        assert_eq!(in_occ, Some(vec![1]));

        let in_occ = index.find("the");
        assert_eq!(in_occ, Some(vec![2, 8, 9]));
    }
}
