//! Unicode character scanner.
use std::str::CharIndices;

pub const EOF_CHAR: char = '\0';

/// A unicode character scanner that iterates over the characters of a source string.
pub struct Cursor<'src> {
    chars: CharIndices<'src>,
    /// Previous character returned by the internal iterator.
    ///
    /// Store the result of the previous iteration so it's
    /// available on demand as the "current" state of the cursor.
    ///
    /// First tuple field is the unicode character position.
    prev: Option<(usize, char)>,
    /// Original source code text.
    source: &'src str,
}

impl<'src> Cursor<'src> {
    /// Construct cursor from given source code text.
    pub fn new(source: &'src str) -> Self {
        Cursor {
            chars: source.char_indices(),
            prev: Some((0, EOF_CHAR)),
            source,
        }
    }

    /// Original source code text.
    pub fn source(&self) -> &'src str {
        self.source
    }

    pub fn rest(&self) -> &'src str {
        match self.try_pos() {
            Some(pos) => &self.source[pos..],
            None => "",
        }
    }

    /// Current unicode position and character in the iteration.
    ///
    /// If iteration has not started, will return end-of-file character.
    #[inline]
    pub fn current(&self) -> (usize, char) {
        match self.prev {
            Some((pos, ch)) => (pos, ch),
            None => panic!("cursor reached end-of-file"),
        }
    }

    #[inline]
    pub fn try_current(&self) -> Option<(usize, char)> {
        self.prev
    }

    /// Current position of unicode character in the iteration.
    #[inline]
    pub fn pos(&self) -> usize {
        self.current().0
    }

    /// Current position of unicode character in the iteration.
    #[inline]
    pub fn try_pos(&self) -> Option<usize> {
        self.prev.map(|(pos, _)| pos)
    }

    /// Current unicode character in the iteration.
    #[inline]
    pub fn char(&self) -> char {
        self.current().1
    }

    /// Current unicode character in the iteration.
    #[inline]
    pub fn try_char(&self) -> Option<char> {
        self.prev.map(|(_, ch)| ch)
    }

    /// Peek the next character without advancing the cursor.
    #[inline]
    pub fn peek_char(&self) -> Option<char> {
        let mut iter = self.chars.clone();
        iter.next().map(|(_, ch)| ch)
    }

    /// Peek the byte position of the next character.
    pub fn peek_offset(&self) -> usize {
        // Byte position of next character is determined by number
        // of bytes taken up by the current character.
        //
        // Because of UTF-8 encoding, there is no easy way
        // to know the size of the current character except
        // advancing the iterator.
        let mut iter = self.chars.clone();
        iter.next()
            .map(|(pos, _)| pos)
            .unwrap_or_else(|| self.source.len())
    }

    /// Indicates whether the cursor is at the end of the source.
    pub fn at_end(&self) -> bool {
        // The iterator may be exhausted, there could be a previous
        // character stored in the state.
        matches!(self.prev, None)
    }

    /// Advances the cursor to the next character.
    ///
    /// Returns `None` if the cursor is end-of-file.
    pub fn bump(&mut self) -> Option<(usize, char)> {
        match self.chars.next() {
            Some((pos, ch)) => {
                // Convert index to smaller integer so
                // tuple fits into 64-bits.
                self.prev = Some((pos, ch));
                Some((pos, ch))
            }
            None => {
                // Point the internal byte offset to one
                // element after the source text, so calls
                // to `offset` and `current` show that the
                // cursor is exhausted.
                self.prev = None;
                None
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_peek() {
        let mut cursor = Cursor::new("abcd");
        assert_eq!(cursor.peek_char(), Some('a'));

        assert_eq!(cursor.bump(), Some((0, 'a')));
        assert_eq!(cursor.bump(), Some((1, 'b')));

        assert_eq!(cursor.peek_char(), Some('c'));

        assert_eq!(cursor.bump(), Some((2, 'c')));

        assert_eq!(cursor.peek_char(), Some('d'));

        assert_eq!(cursor.bump(), Some((3, 'd')));

        assert_eq!(cursor.peek_char(), None);
    }

    #[test]
    fn test_eof() {
        assert_eq!(Cursor::new("").at_end(), false);
        assert_eq!(Cursor::new("abc").at_end(), false);

        // Exhausted cursor must return EOF
        let mut cursor = Cursor::new("a");
        // Initial state
        assert_eq!(cursor.char(), EOF_CHAR);
        assert_eq!(cursor.pos(), 0);
        cursor.bump();
        assert_eq!(cursor.char(), 'a');
        assert_eq!(cursor.pos(), 0);
        cursor.bump();
        assert_eq!(cursor.try_char(), None);
        assert_eq!(cursor.try_pos(), None);

        // Test case where string has explicit EOF sentinal.
        let mut cursor = Cursor::new("abc\0");
        assert_eq!(cursor.bump(), Some((0, 'a')));
        assert_eq!(cursor.char(), 'a');
        assert_eq!(cursor.pos(), 0);

        assert_eq!(cursor.bump(), Some((1, 'b')));
        assert_eq!(cursor.char(), 'b');
        assert_eq!(cursor.pos(), 1);

        assert_eq!(cursor.bump(), Some((2, 'c')));
        assert_eq!(cursor.char(), 'c');
        assert_eq!(cursor.pos(), 2);

        assert_eq!(cursor.bump(), Some((3, EOF_CHAR)));
        assert_eq!(cursor.char(), EOF_CHAR); // explicit
        assert_eq!(cursor.pos(), 3);

        assert_eq!(cursor.bump(), None);
        assert_eq!(cursor.try_char(), None); // implicit
        assert_eq!(cursor.try_pos(), None);
    }
}
