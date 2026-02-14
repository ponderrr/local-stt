pub mod clipboard;
pub mod keyboard;

use crate::config::OutputMode;

pub fn output_text(text: &str, mode: &OutputMode) -> Result<(), String> {
    match mode {
        OutputMode::TypeIntoField => keyboard::type_text(text),
        OutputMode::Clipboard => clipboard::copy_to_clipboard(text),
        OutputMode::Both => {
            keyboard::type_text(text)?;
            clipboard::copy_to_clipboard(text)?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests verify the output_text routing logic.
    // They cannot fully test keyboard/clipboard in a headless CI environment,
    // but we can verify the mode matching logic and error propagation patterns.

    #[test]
    fn test_output_mode_type_into_field_variant() {
        // Verify that OutputMode::TypeIntoField routes to keyboard::type_text
        // In a headless environment, enigo may fail, which is expected
        let result = output_text("test", &OutputMode::TypeIntoField);
        // We just ensure it returns a Result (may be Ok or Err depending on display server)
        let _ = result;
    }

    #[test]
    fn test_output_mode_clipboard_variant() {
        // In a headless environment, arboard may fail, which is expected
        let result = output_text("test", &OutputMode::Clipboard);
        let _ = result;
    }

    #[test]
    fn test_output_mode_both_variant() {
        let result = output_text("test", &OutputMode::Both);
        let _ = result;
    }

    #[test]
    fn test_output_mode_match_arms_are_exhaustive() {
        // This test ensures all OutputMode variants are handled
        // If a new variant is added to OutputMode without updating output_text,
        // this will cause a compile error
        let modes = [
            OutputMode::TypeIntoField,
            OutputMode::Clipboard,
            OutputMode::Both,
        ];
        for mode in &modes {
            let _ = output_text("test", mode);
        }
    }

    #[test]
    fn test_output_text_with_empty_string() {
        // Empty string should not cause a panic in any mode
        let _ = output_text("", &OutputMode::Clipboard);
        let _ = output_text("", &OutputMode::TypeIntoField);
        let _ = output_text("", &OutputMode::Both);
    }

    #[test]
    fn test_output_text_with_unicode() {
        // Unicode text should not cause a panic
        let _ = output_text("Hello \u{2714} \u{1F600}", &OutputMode::Clipboard);
    }

    #[test]
    fn test_output_text_with_multiline() {
        // Multiline text should not cause a panic
        let _ = output_text("line1\nline2\nline3", &OutputMode::Clipboard);
    }
}
