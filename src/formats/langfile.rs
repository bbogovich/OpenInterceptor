//! # The original's language files (`Text/LangFile1.txt` and friends)
//!
//! Plain ASCII text, one string per line:
//!
//! ```text
//! / a comment line starts with a slash
//! S_XCOM          "X-COM"
//! SOME_KEY        "First line~second line"
//! ```
//!
//! A key, whitespace (tabs or spaces), then the text between the first and last double
//! quote on the line; `~` is a line break. Anything else — comments, blank lines,
//! decoration — is skipped. The engine reads these files from the player's CD at run
//! time (`imports:` in a pack's `text/` files); their text is never copied into a pack.

/// The `(key, text)` pairs of a language file, in file order. `~` becomes `\n`.
pub fn parse(bytes: &[u8]) -> Vec<(String, String)> {
    // The files are ASCII; decode leniently so a stray byte cannot fail the whole file
    // (`from_utf8_lossy` borrows when the bytes are valid, and copies only otherwise).
    let text = String::from_utf8_lossy(bytes);
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim_end_matches('\r');
        let Some(first) = line.find('"') else { continue };
        let key = line[..first].trim();
        // A key is one word of letters, digits and underscores starting the line.
        let is_key = !key.is_empty()
            && !line.starts_with(char::is_whitespace)
            && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            && !key.starts_with(|c: char| c.is_ascii_digit());
        if !is_key {
            continue;
        }
        let last = line.rfind('"').unwrap_or(first);
        let value = if last > first { &line[first + 1..last] } else { &line[first + 1..] };
        out.push((key.to_string(), value.replace('~', "\n")));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_text_and_line_breaks() {
        let src = b"/ comment \"not a key\"\r\nS_ONE\t\t\"One\"\r\n\r\nTWO_2   \"Line~break\"\r\n  INDENTED \"no\"\r\n9BAD \"no\"\r\nQUOTE \"say \"hi\" now\"\r\n";
        let got = parse(src);
        assert_eq!(
            got,
            vec![
                ("S_ONE".to_string(), "One".to_string()),
                ("TWO_2".to_string(), "Line\nbreak".to_string()),
                ("QUOTE".to_string(), "say \"hi\" now".to_string()),
            ]
        );
    }
}
