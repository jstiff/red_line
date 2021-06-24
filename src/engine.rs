use crate::line_buffer::LineBuffer;

pub struct Engine {
    line_buffer: LineBuffer,
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            line_buffer: LineBuffer::new(),
        }
    }

    pub fn set_insertion_point(&mut self, pos: usize) {
        self.line_buffer.set_insertion_point(pos)
    }

    pub fn get_insertion_point(&self) -> usize {
        self.line_buffer.get_insertion_point()
    }

    pub fn get_buffer(&self) -> &str {
        &self.line_buffer.get_buffer()
    }

    pub fn set_buffer(&mut self, buffer: String) {
        self.line_buffer.set_buffer(buffer);
    }

    pub fn move_to_end(&mut self) -> usize {
        self.line_buffer.move_to_end()
    }

    // fn get_grapheme_indices(&self) -> Vec<(usize, &str)> {
    //     UnicodeSegmentation::grapheme_indices(self.buffer.as_str(), true).collect()
    // }

    pub fn inc_insertion_point(&mut self) {
        self.line_buffer.inc_insertion_point()
    }

    pub fn dec_insertion_point(&mut self) {
        self.line_buffer.dec_insertion_point()
    }

    pub fn get_buffer_len(&self) -> usize {
        self.line_buffer.get_buffer_len()
    }

    pub fn insert_char(&mut self, pos: usize, c: char) {
        self.line_buffer.insert_char(pos, c)
    }

    pub fn remove_char(&mut self, pos: usize) -> char {
        self.line_buffer.remove_char(pos)
    }
    pub fn insert_str(&mut self, idx: usize, string: &str) {
        self.line_buffer.insert_str(idx, string)
    }

    pub fn is_empty(&self) -> bool {
        self.line_buffer.is_empty()
    }

    pub fn pop(&mut self) -> Option<char> {
        self.line_buffer.pop()
    }

    pub fn clear(&mut self) {
        self.line_buffer.clear()
    }

    pub fn clear_to_end(&mut self) {
        self.line_buffer.clear_to_end()
    }
    pub fn clear_to_insertion_point(&mut self) {
        self.line_buffer.clear_to_insertion_point()
    }

    pub fn clear_range<R>(&mut self, range: R)
    where
        R: std::ops::RangeBounds<usize>,
    {
        self.line_buffer.clear_range(range)
    }

    pub fn move_word_left(&mut self) -> usize {
        self.line_buffer.move_word_left()
    }

    pub fn move_word_right(&mut self) -> usize {
        self.line_buffer.move_word_right()
    }
}
