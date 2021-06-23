use unicode_segmentation::UnicodeSegmentation;

pub struct LineBuffer {
    buffer: String,
    insertion_point: usize,
}

impl LineBuffer {
    pub fn new() -> LineBuffer {
        LineBuffer {
            buffer: String::new(),
            insertion_point: 0,
        }
    }

    pub fn set_insertion_point(&mut self, pos: usize) {
        self.insertion_point = pos;
    }

    pub fn get_insertion_point(&self) -> usize {
        self.insertion_point
    }

    pub fn get_buffer(&self) -> &str {
        &self.buffer
    }

    pub fn set_buffer(&mut self, buffer: String) {
        self.buffer = buffer;
    }

    pub fn move_to_end(&mut self) -> usize {
        self.insertion_point = self.buffer.len();

        self.insertion_point
    }

    // fn get_grapheme_indices(&self) -> Vec<(usize, &str)> {
    //     UnicodeSegmentation::grapheme_indices(self.buffer.as_str(), true).collect()
    // }

    pub fn inc_insertion_point(&mut self) {
        self.insertion_point = if let Some((i, _)) = self.buffer[self.insertion_point..]
            .grapheme_indices(true)
            .nth(1)
        {
            self.insertion_point + i
        } else {
            self.buffer.len()
        };
    }

    pub fn dec_insertion_point(&mut self) {
        self.insertion_point = if let Some((i, _)) = self.buffer[..self.insertion_point]
            .grapheme_indices(true)
            .last()
        {
            i
        } else {
            0
        };
    }

    pub fn get_buffer_len(&self) -> usize {
        self.buffer.len()
    }

    pub fn insert_char(&mut self, pos: usize, c: char) {
        self.buffer.insert(pos, c)
    }

    pub fn remove_char(&mut self, pos: usize) -> char {
        self.buffer.remove(pos)
    }
    pub fn insert_str(&mut self, idx: usize, string: &str) {
        self.buffer.insert_str(idx, string)
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn pop(&mut self) -> Option<char> {
        let result = self.buffer.pop();
        self.insertion_point = self.buffer.len();
        result
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.insertion_point = 0;
    }

    pub fn clear_to_end(&mut self) {
        self.buffer.truncate(self.insertion_point);
    }
    pub fn clear_to_insertion_point(&mut self) {
        self.clear_range(..self.insertion_point);
        self.insertion_point = 0;
    }

    pub fn clear_range<R>(&mut self, range: R)
    where
        R: std::ops::RangeBounds<usize>,
    {
        self.buffer.replace_range(range, "");
    }

    // pub fn get_grapheme_index_left(&self) -> usize {
    //     let grapheme_indices = self.get_grapheme_indices();

    //     let mut prev = 0;
    //     for (idx, _) in grapheme_indices {
    //         if idx >= self.insertion_point {
    //             return prev;
    //         }
    //         prev = idx;
    //     }

    //     prev
    // }

    // pub fn get_grapheme_index_right(&self) -> usize {
    //     let grapheme_indices = self.get_grapheme_indices();

    //     let mut next = self.buffer.len();
    //     for (idx, _) in grapheme_indices.iter().rev() {
    //         if *idx <= self.insertion_point {
    //             return next;
    //         }
    //         next = *idx;
    //     }

    //     next
    // }

    pub fn move_word_left(&mut self) -> usize {
        let mut words = self.buffer[..self.insertion_point]
            .split_word_bound_indices()
            .filter(|(_, word)| !is_word_boundary(word));

        match words.next_back() {
            Some((index, _)) => {
                self.insertion_point = index;
            }
            None => {
                self.insertion_point = 0;
            }
        }

        self.insertion_point
    }

    pub fn move_word_right(&mut self) -> usize {
        let mut words = self.buffer[self.insertion_point..]
            .split_word_bound_indices()
            .filter(|(_, word)| !is_word_boundary(word));

        match words.next() {
            Some((offset, word)) => {
                // Move the insertion point just past the end of the next word
                self.insertion_point += offset + word.len();
            }
            None => {
                self.insertion_point = self.buffer.len();
            }
        }

        self.insertion_point
    }
}

/// Match any sequence of characters that are considered a word boundary
fn is_word_boundary(s: &str) -> bool {
    !s.chars().any(char::is_alphanumeric)
}

#[test]
fn emoji_test() {
    //TODO
    "😊";
    "🤦🏼‍♂️";
}
