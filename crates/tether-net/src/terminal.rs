use vt100::Parser;

pub struct TerminalScreen {
    parser: Parser,
    rows: u16,
    cols: u16,
}

impl TerminalScreen {
    pub fn new(rows: u16, cols: u16) -> Self {
        Self {
            parser: Parser::new(rows, cols, 1000), // 1000 lines scrollback
            rows,
            cols,
        }
    }

    pub fn process(&mut self, bytes: &[u8]) {
        self.parser.process(bytes);
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        self.rows = rows;
        self.cols = cols;
        self.parser.screen_mut().set_size(rows, cols);
    }

    pub fn cursor_position(&self) -> (u16, u16) {
        self.parser.screen().cursor_position()
    }

    pub fn is_cursor_visible(&self) -> bool {
        !self.parser.screen().hide_cursor()
    }

    /// Returns rendered lines as strings
    pub fn render_rows(&self) -> Vec<String> {
        let screen = self.parser.screen();
        let (rows, cols) = screen.size();
        let mut lines = Vec::with_capacity(rows as usize);

        for row in 0..rows {
            let mut line_chars = Vec::with_capacity(cols as usize);
            for col in 0..cols {
                if let Some(cell) = screen.cell(row, col) {
                    let ch = cell.contents();
                    if ch.is_empty() {
                        line_chars.push(' ');
                    } else {
                        line_chars.extend(ch.chars());
                    }
                } else {
                    line_chars.push(' ');
                }
            }
            // Trim right whitespace for neat rendering
            let line_str: String = line_chars.into_iter().collect();
            lines.push(line_str.trim_end().to_string());
        }

        lines
    }

    /// Returns entire contents as text
    pub fn contents(&self) -> String {
        self.parser.screen().contents()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_screen_ansi() {
        let mut term = TerminalScreen::new(24, 80);
        term.process(b"Hello \x1b[32mWorld\x1b[0m!\r\nLine 2");

        let rows = term.render_rows();
        assert!(rows[0].contains("Hello World!"));
        assert_eq!(rows[1], "Line 2");
    }

    #[test]
    fn test_cursor_tracking() {
        let mut term = TerminalScreen::new(24, 80);
        term.process(b"abc");
        let (row, col) = term.cursor_position();
        assert_eq!(row, 0);
        assert_eq!(col, 3);
    }
}
