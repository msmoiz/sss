mod index;

fn main() {
    let pattern = "abc";
    let text = "abcdefg";

    println!("{}", naive::contains(pattern, text));
    println!("{}", rabin_karp::contains(pattern, text));
    println!("{}", boyer_moore::contains(pattern, text));
    println!("{}", knuth_morris_pratt::contains(pattern, text));
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

    #[test]
    fn boyer_moore() {
        test_matcher(super::boyer_moore::contains);
    }

    #[test]
    fn knuth_morris_pratt() {
        test_matcher(super::knuth_morris_pratt::contains);
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
}

mod boyer_moore {
    use std::{cmp::max, collections::HashMap};

    /// Boyer-Moore string search starts comparison from the back of the pattern
    /// and uses heuristics to jump several characters at a time for each
    /// mismatch. It preprocesses the pattern using two rules to determine how
    /// much to shift based on the length of the match before failure: the
    /// bad-character rule and the good-suffix rule.
    ///
    /// The bad-character rule focuses on the character in the text that failed
    /// to match. If it is not present in the pattern, then we can skip the full
    /// pattern length (since the match must occur after that character has been
    /// passed). If it is present in the pattern to the left of the mismatched
    /// position, then we can align the text occurrence and the pattern
    /// occurrence. This page has a good explanation of the bad-character rule:
    /// https://hyperskill.org/learn/step/35869.
    ///
    /// The good-suffix rule focuses on the characters that are matched. If that
    /// suffix repeats itself in the pattern, then we can align the repetition
    /// with the text. We do this only when the repetition is at the beginning
    /// of the pattern or when the character preceding the repetition is not the
    /// same as the character that precedes the suffix (otherwise, the shift
    /// would fail again for the same reason). If the suffix does not repeat
    /// itself in the pattern, then we look for the longest suffix of the suffix
    /// that is also a prefix of the pattern and align on the prefix. If neither
    /// rule matches, we skip the full pattern length (since the suffix will not
    /// be found in the rest of the pattern). This page has a good explanation
    /// of the good-suffix rule: https://hyperskill.org/learn/step/36987.
    ///
    /// The resulting algorithm runs in linear time in the average case, though
    /// it can decay to quadratic time as O(mn).
    pub fn contains(pattern: &str, text: &str) -> bool {
        let pattern: Vec<char> = pattern.chars().collect();
        let text: Vec<char> = text.chars().collect();

        if pattern.is_empty() {
            return true;
        }

        if text.is_empty() || text.len() < pattern.len() {
            return false;
        }

        let bad_character_table = bad_character_table(&pattern);
        let good_suffix_table = good_suffix_table(&pattern);

        let mut i = pattern.len() - 1;

        while i < text.len() {
            let mut j = pattern.len() - 1;
            while j != 0 && text[i] == pattern[j] {
                i -= 1;
                j -= 1;
            }

            if j == 0 {
                return true;
            }

            let bad_char_shift = *bad_character_table.get(&text[i]).unwrap_or(&pattern.len());
            let good_suffix_shift = good_suffix_table[pattern.len() - j - 1];
            i += max(bad_char_shift, good_suffix_shift);
        }

        false
    }

    fn bad_character_table(pattern: &[char]) -> HashMap<char, usize> {
        let mut table = HashMap::new();
        for i in 1..pattern.len() {
            table.insert(pattern[i], pattern.len() - i - 1);
        }
        table
    }

    fn good_suffix_table(pattern: &[char]) -> Vec<usize> {
        let mut table = vec![1]; // shift 1 if no matched suffix

        for suffix_len in 1..pattern.len() {
            let suffix = &pattern[pattern.len() - suffix_len..];
            let mismatch = pattern[pattern.len() - suffix_len - 1];
            let remainder = &pattern[..pattern.len() - 1];

            table.push(pattern.len());

            let mut found_full_suffix = false;

            // try to find next occurrence of full suffix
            for pos in 0..remainder.len() - suffix.len() + 1 {
                if &remainder[pos..pos + suffix_len] == suffix {
                    if pos == 0 || remainder[pos - 1] != mismatch {
                        table[suffix_len] = pattern.len() - pos;
                        found_full_suffix = true;
                    }
                }
            }

            if found_full_suffix {
                continue;
            }

            // try to find longest partial suffix that matches prefix
            for par_suffix_len in (1..suffix_len).rev() {
                let prefix = &pattern[..par_suffix_len];
                let par_suffix = &pattern[pattern.len() - par_suffix_len..];
                if prefix == par_suffix {
                    table[suffix_len] = pattern.len() - par_suffix_len + suffix_len;
                    break;
                }
            }
        }

        table
    }

    #[test]
    fn bad_character_table_correct() {
        let pattern: Vec<char> = "abac".chars().collect();
        let table = bad_character_table(&pattern);
        assert_eq!(table, HashMap::from([('a', 1), ('b', 2), ('c', 0)]));
    }

    #[test]
    fn good_suffix_table_correct() {
        let pattern: Vec<char> = "bcacbcbc".chars().collect();
        let table = good_suffix_table(&pattern);
        assert_eq!(table, vec![1, 5, 8, 5, 10, 11, 12, 13]);
    }
}

mod knuth_morris_pratt {
    /// Knuth-Morris-Pratt string search achieves linear time complexity by
    /// preprocessing the pattern to determine how much of the pattern to
    /// reevalaute once a mismatch is found. The text cursor only moves forward,
    /// meaning each text character is only evaluated once.
    ///
    /// The partial match table specifies the amount to backtrack the pattern
    /// cursor. If the backtrack value is -1, we do not backtrack at all but
    /// instead advance both cursors. If the backtrack value is positive, set
    /// the pattern cursor to the backtrack value. The Wikipedia page for the
    /// algorithm has a useful reference implementation:
    /// https://en.wikipedia.org/wiki/Knuth%E2%80%93Morris%E2%80%93Pratt_algorithm.
    pub fn contains(pattern: &str, text: &str) -> bool {
        let pattern: Vec<char> = pattern.chars().collect();
        let text: Vec<char> = text.chars().collect();

        if pattern.is_empty() {
            return true;
        }

        if text.is_empty() || text.len() < pattern.len() {
            return false;
        }

        let partial_match_table = partial_match_table(&pattern);

        let mut i = 0;
        let mut j = 0;
        while i < text.len() {
            if text[i] == pattern[j] {
                i += 1;
                j += 1;

                if j == pattern.len() {
                    return true;
                }
            } else {
                let k = partial_match_table[j];
                if k < 0 {
                    i += 1;
                    j = (k + 1) as usize;
                } else {
                    j = k as usize;
                }
            }
        }

        false
    }

    fn partial_match_table(pattern: &[char]) -> Vec<isize> {
        let mut table = vec![-1]; // no shift if there is no match
        let mut cnd = 0;
        for i in 1..pattern.len() {
            if pattern[i] == pattern[cnd as usize] {
                table.push(table[cnd as usize]);
            } else {
                table.push(cnd);
                while cnd >= 0 && pattern[i] != pattern[cnd as usize] {
                    cnd = table[cnd as usize];
                }
            }
            cnd += 1;
        }
        table
    }

    #[test]
    fn partial_match_table_correct() {
        let pattern: Vec<char> = "abcdabd".chars().collect();
        let table = partial_match_table(&pattern);
        assert_eq!(table, vec![-1, 0, 0, 0, -1, 0, 2]);
    }
}
