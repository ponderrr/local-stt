//! LocalAgreement-2 deduplication for streaming Whisper output.
//! Compares consecutive inference passes at the word level and confirms
//! words that appear identically in the same position across two passes.

/// Result of processing one inference pass through the agreement algorithm.
#[derive(Debug, Clone, PartialEq)]
pub struct AgreementResult {
    /// Words newly confirmed in this pass (empty string if none).
    pub newly_confirmed: String,
    /// Tentative words that may change in the next pass.
    pub tentative: String,
}

/// LocalAgreement-2 streaming deduplication.
///
/// Tracks consecutive Whisper outputs and confirms words that appear
/// at the same position in two consecutive passes. Confirmed words are
/// permanent — they never change or disappear.
pub struct LocalAgreement {
    prev_words: Vec<String>,
    confirmed_count: usize,
    /// All confirmed text so far (for spacing logic).
    confirmed_text: String,
}

impl LocalAgreement {
    pub fn new() -> Self {
        Self {
            prev_words: Vec::new(),
            confirmed_count: 0,
            confirmed_text: String::new(),
        }
    }

    /// Process the full text from one inference pass.
    /// Returns newly confirmed words and the tentative tail.
    pub fn process(&mut self, current_text: &str) -> AgreementResult {
        let curr_words: Vec<String> = current_text
            .split_whitespace()
            .map(|w| w.to_string())
            .collect();

        if self.prev_words.is_empty() {
            // First pass — nothing to compare, everything is tentative
            self.prev_words = curr_words;
            return AgreementResult {
                newly_confirmed: String::new(),
                tentative: current_text.trim().to_string(),
            };
        }

        // Find longest common prefix starting from confirmed_count
        let mut lcp_end = self.confirmed_count;
        while lcp_end < curr_words.len() && lcp_end < self.prev_words.len() {
            if Self::words_match(&curr_words[lcp_end], &self.prev_words[lcp_end]) {
                lcp_end += 1;
            } else {
                break;
            }
        }

        // Words from confirmed_count..lcp_end are newly confirmed
        let newly_confirmed_words: Vec<&str> = curr_words[self.confirmed_count..lcp_end]
            .iter()
            .map(|w| w.as_str())
            .collect();
        let newly_confirmed = newly_confirmed_words.join(" ");

        if !newly_confirmed.is_empty() {
            if !self.confirmed_text.is_empty() {
                self.confirmed_text.push(' ');
            }
            self.confirmed_text.push_str(&newly_confirmed);
        }

        self.confirmed_count = lcp_end;

        // Everything after confirmed_count in current output is tentative
        let tentative_words: Vec<&str> = curr_words[lcp_end..].iter().map(|w| w.as_str()).collect();
        let tentative = tentative_words.join(" ");

        self.prev_words = curr_words;

        AgreementResult {
            newly_confirmed,
            tentative,
        }
    }

    /// Finalize: confirm all remaining tentative words.
    /// Call on EndOfSpeech. Returns the remaining words as confirmed.
    pub fn finalize(&mut self) -> String {
        let remaining: Vec<&str> = self.prev_words[self.confirmed_count..]
            .iter()
            .map(|w| w.as_str())
            .collect();
        let result = remaining.join(" ");

        if !result.is_empty() && !self.confirmed_text.is_empty() {
            self.confirmed_text.push(' ');
        }
        self.confirmed_text.push_str(&result);

        let confirmed = self.confirmed_text.clone();
        self.reset();
        // Return only the remaining (not-yet-confirmed) words
        let _ = confirmed; // confirmed_text was used for spacing tracking
        result
    }

    /// Reset all state for a new utterance.
    pub fn reset(&mut self) {
        self.prev_words.clear();
        self.confirmed_count = 0;
        self.confirmed_text.clear();
    }

    /// Compare two words for agreement. Normalizes by lowercasing and
    /// stripping trailing punctuation so "Hello," matches "Hello".
    fn words_match(a: &str, b: &str) -> bool {
        Self::normalize(a) == Self::normalize(b)
    }

    /// Normalize a word for comparison: lowercase, strip trailing punctuation.
    fn normalize(word: &str) -> String {
        let lower = word.to_lowercase();
        lower
            .trim_end_matches(|c: char| c.is_ascii_punctuation())
            .to_string()
    }

    /// Get all confirmed text so far.
    pub fn confirmed_text(&self) -> &str {
        &self.confirmed_text
    }
}

impl Default for LocalAgreement {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Basic Agreement ---

    #[test]
    fn test_first_pass_everything_tentative() {
        let mut la = LocalAgreement::new();
        let result = la.process("Hello how are you");
        assert_eq!(result.newly_confirmed, "");
        assert_eq!(result.tentative, "Hello how are you");
    }

    #[test]
    fn test_second_pass_confirms_matching_prefix() {
        let mut la = LocalAgreement::new();
        la.process("Hello how are");
        let result = la.process("Hello how are you");
        assert_eq!(result.newly_confirmed, "Hello how are");
        assert_eq!(result.tentative, "you");
    }

    #[test]
    fn test_third_pass_confirms_incrementally() {
        let mut la = LocalAgreement::new();
        la.process("Hello how are");
        la.process("Hello how are you");
        let result = la.process("Hello how are you doing");
        assert_eq!(result.newly_confirmed, "you");
        assert_eq!(result.tentative, "doing");
    }

    #[test]
    fn test_confirmed_count_tracks_correctly() {
        let mut la = LocalAgreement::new();
        la.process("Hello world");
        la.process("Hello world foo");
        assert_eq!(la.confirmed_count, 2);
    }

    // --- Finalize ---

    #[test]
    fn test_finalize_returns_remaining_tentative() {
        let mut la = LocalAgreement::new();
        la.process("Hello how are");
        la.process("Hello how are you");
        let remaining = la.finalize();
        assert_eq!(remaining, "you");
    }

    #[test]
    fn test_finalize_with_nothing_tentative() {
        let mut la = LocalAgreement::new();
        la.process("Hello");
        la.process("Hello");
        let remaining = la.finalize();
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_finalize_resets_state() {
        let mut la = LocalAgreement::new();
        la.process("Hello world");
        la.process("Hello world");
        la.finalize();
        assert_eq!(la.confirmed_count, 0);
        assert!(la.prev_words.is_empty());
    }

    // --- Reset ---

    #[test]
    fn test_reset_clears_all_state() {
        let mut la = LocalAgreement::new();
        la.process("Hello world");
        la.process("Hello world foo");
        la.reset();
        assert_eq!(la.confirmed_count, 0);
        assert!(la.prev_words.is_empty());
        assert!(la.confirmed_text.is_empty());
    }

    #[test]
    fn test_after_reset_behaves_like_new() {
        let mut la = LocalAgreement::new();
        la.process("old text");
        la.process("old text here");
        la.reset();

        let result = la.process("New utterance");
        assert_eq!(result.newly_confirmed, "");
        assert_eq!(result.tentative, "New utterance");
    }

    // --- Word Normalization ---

    #[test]
    fn test_punctuation_normalized_for_matching() {
        let mut la = LocalAgreement::new();
        la.process("Hello, world");
        let result = la.process("Hello world today");
        assert_eq!(result.newly_confirmed, "Hello world");
    }

    #[test]
    fn test_case_normalized_for_matching() {
        let mut la = LocalAgreement::new();
        la.process("hello World");
        let result = la.process("Hello world today");
        assert_eq!(result.newly_confirmed, "Hello world");
    }

    // --- Edge Cases ---

    #[test]
    fn test_empty_text() {
        let mut la = LocalAgreement::new();
        let result = la.process("");
        assert_eq!(result.newly_confirmed, "");
        assert_eq!(result.tentative, "");
    }

    #[test]
    fn test_single_word_confirms() {
        let mut la = LocalAgreement::new();
        la.process("Hello");
        let result = la.process("Hello world");
        assert_eq!(result.newly_confirmed, "Hello");
        assert_eq!(result.tentative, "world");
    }

    #[test]
    fn test_complete_change_confirms_nothing() {
        let mut la = LocalAgreement::new();
        la.process("Hello world");
        let result = la.process("Goodbye earth");
        assert_eq!(result.newly_confirmed, "");
        assert_eq!(result.tentative, "Goodbye earth");
    }

    #[test]
    fn test_shorter_output_than_previous() {
        let mut la = LocalAgreement::new();
        la.process("Hello world foo bar");
        let result = la.process("Hello world");
        assert_eq!(result.newly_confirmed, "Hello world");
        assert_eq!(result.tentative, "");
    }

    #[test]
    fn test_identical_consecutive_passes() {
        let mut la = LocalAgreement::new();
        la.process("Hello world");
        let result = la.process("Hello world");
        assert_eq!(result.newly_confirmed, "Hello world");
        assert_eq!(result.tentative, "");
    }

    #[test]
    fn test_confirmed_text_accumulates() {
        let mut la = LocalAgreement::new();
        la.process("Hello how");
        la.process("Hello how are");
        assert_eq!(la.confirmed_text(), "Hello how");

        la.process("Hello how are you");
        assert_eq!(la.confirmed_text(), "Hello how are");
    }

    #[test]
    fn test_many_passes_progressive_confirmation() {
        let mut la = LocalAgreement::new();
        la.process("The");
        la.process("The quick");
        la.process("The quick brown");
        la.process("The quick brown fox");
        la.process("The quick brown fox jumps");

        assert_eq!(la.confirmed_count, 4);
        assert_eq!(la.confirmed_text(), "The quick brown fox");
    }

    #[test]
    fn test_finalize_after_many_passes() {
        let mut la = LocalAgreement::new();
        la.process("The");
        la.process("The quick");
        la.process("The quick brown");
        la.process("The quick brown fox");
        la.process("The quick brown fox jumps");
        let remaining = la.finalize();
        assert_eq!(remaining, "jumps");
    }

    // --- Spacing ---

    #[test]
    fn test_newly_confirmed_has_correct_spacing() {
        let mut la = LocalAgreement::new();
        la.process("Hello world");
        let r = la.process("Hello world foo");
        assert_eq!(r.newly_confirmed, "Hello world");
    }

    // --- Whitespace Handling ---

    #[test]
    fn test_extra_whitespace_handled() {
        let mut la = LocalAgreement::new();
        la.process("  Hello   world  ");
        let result = la.process("Hello world foo");
        assert_eq!(result.newly_confirmed, "Hello world");
    }
}
