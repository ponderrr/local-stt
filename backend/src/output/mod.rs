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
