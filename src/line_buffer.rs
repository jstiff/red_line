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
    pub fn inc_insertion_point(&mut self) {
        self.insertion_point += 1
    }
    pub fn dec_insertion_point(&mut self) {
        self.insertion_point -= 1
    }
    pub fn get_buffer(&self) -> &str {
        &self.buffer
    }
    pub fn get_buffer_len(&self) -> usize {
        self.buffer.len()
    }
    pub fn slice_buffer(&self, pos: usize) -> &str {
        &self.buffer[pos..]
    }
    pub fn insert_char(&mut self, pos: usize, c: char) {
        self.buffer.insert(pos, c)
    }
    pub fn remove_char(&mut self, pos: usize) -> char {
        self.buffer.remove(pos)
    }
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
    pub fn pop(&mut self) -> Option<char> {
        self.buffer.pop()
    }
    pub fn clear(&mut self) {
        self.buffer.clear()
    }
    pub fn move_word_left(&mut self) -> usize {
        match self
            .buffer
            .rmatch_indices(&[' ', '\t'][..])
            .find(|(index, _)| index < &(self.insertion_point - 1))
        {
            Some((index, _)) => {
                self.insertion_point = index + 1;
            }
            None => {
                self.insertion_point = 0;
            }
        }
        self.insertion_point
    }
    pub fn move_word_right(&mut self) -> usize {
        match self
            .buffer
            .match_indices(&[' ', '\t'][..])
            .find(|(index, _)| index > &(self.insertion_point))
        {
            Some((index, _)) => {
                self.insertion_point = index + 1;
            }
            None => {
                self.insertion_point = self.get_buffer_len();
            }
        }
        self.insertion_point
    }
}
