// Display width of a character in terminal cells, as the unicode-width
// crate (UAX #11, used by alacritty and ratatui) gives it: 0 for control
// characters and combining marks, 2 for East Asian wide/fullwidth and emoji
// presentation, 1 otherwise.

use unicode_width::UnicodeWidthChar;

pub fn char_width(ch: char) -> usize {
    ch.width().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths() {
        assert_eq!(char_width('a'), 1);
        assert_eq!(char_width('한'), 2);
        assert_eq!(char_width('中'), 2);
        assert_eq!(char_width('🚀'), 2);
        assert_eq!(char_width('\u{301}'), 0);
        assert_eq!(char_width('\u{fe0f}'), 0);
        assert_eq!(char_width('─'), 1);
        assert_eq!(char_width('⏺'), 1);
        assert_eq!(char_width('\x1b'), 0);
    }
}
