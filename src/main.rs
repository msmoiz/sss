fn main() {
    let pattern = "abc";
    let text = "abcdefg";

    println!("{}", naive::contains(pattern, text));
    println!("{}", rabin_karp::contains(pattern, text));
}

#[cfg(test)]
mod test {
    pub const TEST_PATTERN: &'static str = "abcde";

    pub const TEST_CASES: [(&'static str, bool); 10] = [
        ("abcdefghij", true),
        ("12345abcde", true),
        ("klabcdefgh", true),
        ("qrabcdefst", true),
        ("vwxyzabcde", true),
        ("ijklmnopab", false),
        ("fghijklmno", false),
        ("pqrstuvwxyz", false),
        ("lmnopqrst", false),
        ("uvwxyzabcd", false),
    ];

    fn test_matcher(matcher: fn(&str, &str) -> bool) {
        for (text, expected) in TEST_CASES {
            let actual = matcher(TEST_PATTERN, text);
            if actual != expected {
                panic!(
                    "expected {} for \"{text}\"",
                    if expected { "match" } else { "no match" }
                );
            }
        }
    }

    #[test]
    fn naive() {
        test_matcher(super::naive::contains);
    }

    #[test]
    fn rabin_karp() {
        test_matcher(super::rabin_karp::contains);
    }
}

mod naive {
    /// Naive string search checks for the presence of a match at each position
    /// of the input text. This requires no additional space but exhibits O(mn)
    /// time complexity in the worst case.
    pub fn contains(pattern: &str, text: &str) -> bool {
        let pattern: Vec<char> = pattern.chars().collect();
        let text: Vec<char> = text.chars().collect();

        if pattern.is_empty() {
            return true;
        }

        if text.is_empty() || text.len() < pattern.len() {
            return false;
        }

        for i in 0..text.len() {
            if contains_inner(&pattern, &text[i..]) {
                return true;
            }
        }

        false
    }

    fn contains_inner(pattern: &[char], text: &[char]) -> bool {
        for (i, p) in pattern.iter().enumerate() {
            if i == text.len() {
                return false;
            }

            if &text[i] != p {
                return false;
            }
        }
        true
    }
}

mod rabin_karp {
    /// Rabin-Karp string search is similar to naive string search in that it
    /// checks for a match at every position of the input text. However, it
    /// skips the check at a given position if the hash of the substring at that
    /// position (of pattern length) does not match the hash of the pattern.
    ///
    /// Computing a hash at a given position typically requires reading every
    /// character in the substring (and would be no better than naive search).
    /// Instead the algorithm makes use of a rolling hash, which allows the hash
    /// to be computed incrementally in constant time for each position. The
    /// following video provides a useful explanation of the rolling hash
    /// mechanism: https://www.youtube.com/watch?v=BfUejqd07yo. The following
    /// post is also useful for the same: https://stackoverflow.com/questions/6109624/
    /// need-help-in-understanding-rolling-hash-computation-in-constant-time-for-rabin-k.
    pub fn contains(pattern: &str, text: &str) -> bool {
        let pattern: Vec<char> = pattern.chars().collect();
        let text: Vec<char> = text.chars().collect();

        if pattern.is_empty() {
            return true;
        }

        if text.is_empty() || text.len() < pattern.len() {
            return false;
        }

        let pattern_hash = RollingHasher::new(&pattern).hash();
        let mut text_hasher = RollingHasher::new(&text[..pattern.len()]);
        for i in 0..text.len() {
            if text[i..].len() < pattern.len() {
                continue;
            }

            if i > 0 {
                let in_ch = text[i + pattern.len() - 1];
                let out_ch = text[i - 1];
                text_hasher.roll(in_ch, out_ch);
            }

            let text_hash = text_hasher.hash();
            if text_hash != pattern_hash {
                continue;
            }

            if contains_inner(&pattern, &text[i..]) {
                return true;
            }
        }

        false
    }

    struct RollingHasher {
        hash: u64,
        window: usize,
    }

    const MULTIPLIER: u64 = 10;
    const MODULO: u64 = 256;

    impl RollingHasher {
        fn new(init: &[char]) -> Self {
            let window = init.len();

            let mut hash = 0;
            for (i, ch) in init.iter().enumerate() {
                let power = (window - i - 1) as u64;
                let next = *ch as u64 * MULTIPLIER.pow(power as u32);
                hash += next;
            }
            hash %= MODULO;

            Self { hash, window }
        }

        fn roll(&mut self, in_ch: char, out_ch: char) {
            let power = (self.window - 1) as u64;
            let previous = ((out_ch as u64) * (MULTIPLIER.pow(power as u32))) % MODULO;
            self.hash = (self.hash + MODULO - previous) % MODULO;
            self.hash = self.hash * MULTIPLIER;

            let next = in_ch as u64;
            self.hash = self.hash + next;
            self.hash %= MODULO;
        }

        fn hash(&self) -> u64 {
            self.hash
        }
    }

    #[test]
    fn rolled_hash_matches_direct_hash() {
        let text: Vec<char> = "abc".chars().collect();
        let mut hasher_a = RollingHasher::new(&text);
        hasher_a.roll('a', 'a');

        let text: Vec<char> = "bca".chars().collect();
        let hasher_b = RollingHasher::new(&text);

        assert_eq!(hasher_a.hash(), hasher_b.hash());
    }

    fn contains_inner(pattern: &[char], text: &[char]) -> bool {
        for (i, p) in pattern.iter().enumerate() {
            if i == text.len() {
                return false;
            }

            if &text[i] != p {
                return false;
            }
        }
        true
    }

    #[test]
    fn match_at_start() {
        let pattern = "abc";
        let text = "abcdefg";
        assert!(contains(pattern, text));
    }

    #[test]
    fn match_elsewhere() {
        let pattern = "abc";
        let text = "defgabc";
        assert!(contains(pattern, text));
    }

    #[test]
    fn reject_not_present() {
        let pattern = "abc";
        let text = "abdefg";
        assert!(!contains(pattern, text));
    }
}
