fn main() {
    let pattern = "abc";
    let text = "abcdefg";
    println!("{}", naive::contains(pattern, text));
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

        if text.is_empty() {
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
