use std::io::{stdout, Write};

use crossterm::{
    cursor::{
        position, MoveLeft, MoveRight, MoveToColumn, MoveToNextLine, RestorePosition, SavePosition,
    },
    event::read,
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal, ExecutableCommand, QueueableCommand, Result,
};
use std::collections::VecDeque;
use std::io::Stdout;
mod line_buffer;
use line_buffer::LineBuffer;
const HISTORY_SIZE: usize = 100;

fn print_message(stdout: &mut Stdout, msg: &str) -> Result<()> {
    stdout
        .queue(Print("\n"))?
        .queue(MoveToColumn(1))?
        .queue(Print("Our buffer: "))?
        .queue(Print(&msg))?
        .queue(Print("\n"))?
        .queue(MoveToColumn(1))?;
    stdout.flush()?;

    Ok(())
}
fn buffer_repaint(
    stdout: &mut Stdout,
    buffer: &LineBuffer,
    prompt_offset: u16,
    new_index: usize,
) -> Result<()> {
    let raw_buffer = buffer.get_buffer();
    stdout.queue(MoveToColumn(prompt_offset))?;
    stdout.queue(Print(&raw_buffer[0..new_index]))?;
    stdout.queue(SavePosition)?;
    stdout.queue(Print(&raw_buffer[new_index..]))?;
    stdout.queue(RestorePosition)?;

    Ok(())
}
fn main() -> Result<()> {
    let mut stdout = stdout();
    let mut buffer = LineBuffer::new();
    let mut history = VecDeque::with_capacity(HISTORY_SIZE);
    let mut history_cursor = -1i64;
    let mut has_history = false;

    terminal::enable_raw_mode()?;
    'repl: loop {
        // loop to print out the prompt...

        stdout
            .execute(SetForegroundColor(Color::Blue))?
            .execute(Print("> "))?
            .execute(ResetColor)?;

        // set where the input begins

        let (mut prompt_offset, _) = position()?;
        //println!("prompt offset {:?}", prompt_offset);

        prompt_offset += 1;

        'input: loop {
            match read()? {
                Event::Key(KeyEvent { code, modifiers }) => match code {
                    KeyCode::Char(c) => {
                        if modifiers == KeyModifiers::CONTROL {
                            if c == 'd' {
                                stdout.queue(MoveToNextLine(1))?.queue(Print("Leaving!"))?;
                                break 'repl;
                            }
                        }
                        let insertion_point = buffer.get_insertion_point();

                        if insertion_point == buffer.get_buffer_len() {
                            stdout.queue(Print(c))?;
                        } else {
                            stdout
                                .queue(Print(c))?
                                .queue(Print(buffer.slice_buffer(insertion_point)))?
                                .queue(MoveToColumn(insertion_point as u16 + prompt_offset + 1))?;
                        }
                        stdout.flush()?;
                        buffer.insert_char(buffer.get_insertion_point(), c);
                        buffer.inc_insertion_point();
                    }
                    KeyCode::Backspace => {
                        let insertion_point = buffer.get_insertion_point();
                        if insertion_point == buffer.get_buffer_len() && !buffer.is_empty() {
                            buffer.dec_insertion_point();
                            buffer.pop();
                            stdout
                                .queue(MoveLeft(1))?
                                .queue(Print(" "))?
                                .queue(MoveLeft(1))?;
                            stdout.flush()?;
                        } else if insertion_point < buffer.get_buffer_len() && !buffer.is_empty() {
                            buffer.dec_insertion_point();
                            let insertion_point = buffer.get_insertion_point();
                            buffer.remove_char(insertion_point);

                            stdout
                                .queue(MoveLeft(1))?
                                .queue(Print(buffer.slice_buffer(insertion_point)))?
                                .queue(Print(" "))?
                                .queue(MoveToColumn(insertion_point as u16 + prompt_offset))?;
                            stdout.flush()?;
                        }
                    }
                    KeyCode::Delete => {
                        let insertion_point = buffer.get_insertion_point();
                        if insertion_point < buffer.get_buffer_len() && !buffer.is_empty() {
                            buffer.remove_char(insertion_point);
                            stdout
                                .queue(Print(buffer.slice_buffer(insertion_point)))?
                                .queue(Print(' '))?
                                .queue(MoveToColumn(insertion_point as u16 + prompt_offset))?;
                            stdout.flush()?;
                        }
                    }
                    KeyCode::Home => {
                        stdout.queue(MoveToColumn(prompt_offset))?;
                        stdout.flush()?;
                        buffer.set_insertion_point(0);
                    }
                    KeyCode::End => {
                        let buffer_len = buffer.get_buffer_len();
                        stdout.queue(MoveToColumn(prompt_offset + buffer_len as u16))?;
                        stdout.flush()?;
                        buffer.set_insertion_point(buffer_len);
                    }
                    KeyCode::Enter => {
                        if buffer.get_buffer() == "exit" {
                            break 'repl;
                        } else {
                            if history.len() + 1 == HISTORY_SIZE {
                                // History is "full", so we delete the oldest entry first,
                                // before adding a new one.
                                history.pop_back();
                            }
                            history.push_front(String::from(buffer.get_buffer()));
                            has_history = true;
                            // reset the history cursor - we want to start at the bottom of the
                            // history again.
                            history_cursor = -1;

                            print_message(
                                &mut stdout,
                                &format!("The Buffer: {}", buffer.get_buffer()),
                            )?;
                            buffer.clear();
                            buffer.set_insertion_point(0);
                            break 'input;
                        }
                    }
                    KeyCode::Up => {
                        // Up means: navigate through the history.
                        if has_history && history_cursor < (history.len() as i64 - 1) {
                            history_cursor += 1;
                            let history_entry =
                                history.get(history_cursor as usize).unwrap().clone();
                            let previous_buffer_len = buffer.get_buffer_len();
                            buffer.set_buffer(history_entry.clone());
                            let new_buffer_len = buffer.get_buffer_len();
                            let new_insertion_point = buffer.move_to_end();

                            // After changing the buffer, we also need to repaint the whole
                            // line.
                            // TODO: Centralize painting of the line?!
                            stdout
                                .queue(MoveToColumn(prompt_offset))?
                                .queue(Print(buffer.get_buffer()))?;

                            // Print over the rest of the line with spaces if the typed stuff
                            // was longer than the history entry length
                            for _ in 0..std::cmp::max(
                                0,
                                previous_buffer_len as i64 - new_buffer_len as i64,
                            ) {
                                stdout.queue(Print(" "))?;
                            }
                            stdout
                                .queue(MoveToColumn(new_insertion_point as u16 + prompt_offset))?
                                .flush()?;
                        }
                    }
                    KeyCode::Down => {
                        // Down means: navigate forward through the history. If we reached the
                        // bottom of the history, we clear the buffer, to make it feel like
                        // zsh/bash/whatever
                        if history_cursor >= 0 {
                            history_cursor -= 1;
                        }
                        let new_buffer = if history_cursor < 0 {
                            String::new()
                        } else {
                            // We can be sure that we always have an entry on hand, that's why
                            // unwrap is fine.
                            history.get(history_cursor as usize).unwrap().clone()
                        };

                        let previous_buffer_len = buffer.get_buffer_len();
                        buffer.set_buffer(new_buffer.clone());
                        let new_buffer_len = buffer.get_buffer_len();
                        let new_insertion_point = buffer.move_to_end();

                        // After changing the buffer, we also need to repaint the whole
                        // line.
                        // TODO: Centralize painting of the line?!
                        stdout
                            .queue(MoveToColumn(prompt_offset))?
                            .queue(Print(buffer.get_buffer()))?;

                        // Print over the rest of the line with spaces if the typed stuff
                        // was longer than the history entry length
                        for _ in
                            0..std::cmp::max(0, previous_buffer_len as i64 - new_buffer_len as i64)
                        {
                            stdout.queue(Print(" "))?;
                        }
                        stdout
                            .queue(MoveToColumn(new_insertion_point as u16 + prompt_offset))?
                            .flush()?;
                    }
                    KeyCode::Left => {
                        if buffer.get_insertion_point() > 0 {
                            // If the ALT modifier is set, we want to jump words for more
                            // natural editing. Jumping words basically means: move to next
                            // whitespace in the given direction.
                            if modifiers == KeyModifiers::ALT {
                                let new_insertion_point = buffer.move_word_left();
                                stdout.queue(MoveToColumn(
                                    new_insertion_point as u16 + prompt_offset,
                                ))?;
                            } else {
                                buffer.dec_insertion_point();
                                let idx = buffer.get_insertion_point();
                                buffer_repaint(&mut stdout, &buffer, prompt_offset, idx)?;
                            }

                            stdout.flush()?;
                        }
                    }
                    KeyCode::Right => {
                        if buffer.get_insertion_point() < buffer.get_buffer_len() {
                            if modifiers == KeyModifiers::ALT {
                                let new_insertion_point = buffer.move_word_right();
                                stdout.queue(MoveToColumn(
                                    new_insertion_point as u16 + prompt_offset,
                                ))?;
                            } else {
                                buffer.inc_insertion_point();
                                let idx = buffer.get_insertion_point();
                                buffer_repaint(&mut stdout, &buffer, prompt_offset, idx)?;
                            }

                            stdout.flush()?;
                        }
                    }
                    _ => {}
                },
                Event::Mouse(event) => {
                    print_message(&mut stdout, &format!("{:?}", event))?;
                }
                Event::Resize(width, height) => {
                    print_message(&mut stdout, &format!("width: {} height {} ", width, height))?;
                }
            }
        }
    }

    terminal::disable_raw_mode()?;
    Ok(())
}
