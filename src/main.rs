use std::io::{stdout, Write};
use std::time::Duration;

use crossterm::{
    cursor::{position, MoveToColumn, MoveToNextLine, RestorePosition, SavePosition},
    event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
    ExecutableCommand, QueueableCommand, Result,
};

use std::collections::VecDeque;
use std::io::Stdout;
mod engine;
use engine::{EditCommand, Engine};
mod line_buffer;

const HISTORY_SIZE: usize = 100;

fn print_message(stdout: &mut Stdout, msg: &str) -> Result<()> {
    stdout
        .queue(Print("\n"))?
        .queue(MoveToColumn(1))?
        .queue(Print(msg))?
        .queue(Print("\n"))?
        .queue(MoveToColumn(1))?;
    stdout.flush()?;

    Ok(())
}

fn buffer_repaint(stdout: &mut Stdout, engine: &Engine, prompt_offset: u16) -> Result<()> {
    let raw_buffer = engine.get_buffer();
    let new_index = engine.get_insertion_point();

    // Repaint logic:
    //
    // Start after the prompt
    // Draw the string slice from 0 to the grapheme start left of insertion point
    // Then, get the position on the screen
    // Then draw the remainer of the engine from above
    // Finally, reset the cursor to the saved position

    stdout.queue(MoveToColumn(prompt_offset))?;
    stdout.queue(Print(&raw_buffer[0..new_index]))?;
    stdout.queue(SavePosition)?;
    stdout.queue(Print(&raw_buffer[new_index..]))?;
    stdout.queue(Clear(ClearType::UntilNewLine))?;
    stdout.queue(RestorePosition)?;

    stdout.flush()?;

    Ok(())
}

// this fn is totally ripped off from crossterm's examples
// it's really a diagnostic routine to see if crossterm is
// even seeing the events. if you press a key and no events
// are printed, it's a good chance your terminal is eating
// those events.
fn print_events(stdout: &mut Stdout) -> Result<()> {
    loop {
        // Wait up to 5s for another event
        if poll(Duration::from_millis(5_000))? {
            // It's guaranteed that read() wont block if `poll` returns `Ok(true)`
            let event = read()?;

            // just reuse the print_message fn to show events
            print_message(stdout, &format!("Event::{:?}", event))?;

            // hit the esc key to git out
            if event == Event::Key(KeyCode::Esc.into()) {
                break;
            }
        } else {
            // Timeout expired, no event for 5s
            print_message(stdout, "Waiting for you to type...")?;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let mut stdout = stdout();

    terminal::enable_raw_mode()?;
    // quick command like parameter handling
    let args: Vec<String> = std::env::args().collect();
    // if -k is passed, show the events
    if args.len() > 1 && args[1] == "-k" {
        print_message(&mut stdout, "Ready to print events:")?;
        print_events(&mut stdout)?;
        terminal::disable_raw_mode()?;
        println!();
        return Ok(());
    };

    let mut engine = Engine::new();
    let mut history = VecDeque::with_capacity(HISTORY_SIZE);
    let mut history_cursor = -1i64;
    let mut has_history = false;
    let mut cut_buffer = String::new();

    'repl: loop {
        // print our prompt
        stdout
            .execute(SetForegroundColor(Color::Blue))?
            .execute(Print("> "))?
            .execute(ResetColor)?;

        // set where the input begins
        let (mut prompt_offset, _) = position()?;
        prompt_offset += 1;

        'input: loop {
            match read()? {
                Event::Key(KeyEvent {
                    code,
                    modifiers: KeyModifiers::CONTROL,
                }) => match code {
                    KeyCode::Char('d') => {
                        if engine.get_buffer().is_empty() {
                            stdout.queue(MoveToNextLine(1))?.queue(Print("exit"))?;
                            break 'repl;
                        } else {
                            engine.run_edit_commands(&[EditCommand::Delete]);
                            buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                        }
                    }
                    KeyCode::Char('a') => {
                        engine.run_edit_commands(&[EditCommand::MoveToStart]);
                        buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                    }
                    KeyCode::Char('e') => {
                        engine.run_edit_commands(&[EditCommand::MoveToEnd]);
                        buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                    }
                    KeyCode::Char('k') => {
                        let cut_slice = &engine.get_buffer()[engine.get_insertion_point()..];
                        if !cut_slice.is_empty() {
                            cut_buffer.replace_range(.., cut_slice);
                            engine.clear_to_end();
                            buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                        }
                    }
                    KeyCode::Char('u') => {
                        if engine.get_insertion_point() > 0 {
                            cut_buffer.replace_range(
                                ..,
                                &engine.get_buffer()[..engine.get_insertion_point()],
                            );
                            engine.clear_to_insertion_point();
                            buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                        }
                    }
                    KeyCode::Char('y') => {
                        engine.insert_str(engine.get_insertion_point(), &cut_buffer);
                        engine.set_insertion_point(engine.get_insertion_point() + cut_buffer.len());
                        buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                    }
                    KeyCode::Char('b') => {
                        engine.dec_insertion_point();
                        buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                    }
                    KeyCode::Char('f') => {
                        engine.inc_insertion_point();
                        buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                    }
                    KeyCode::Char('c') => {
                        engine.clear();
                        stdout.queue(Print('\n'))?.queue(MoveToColumn(1))?.flush()?;
                        break 'input;
                    }
                    KeyCode::Char('h') => {
                        let insertion_point = engine.get_insertion_point();
                        if insertion_point == engine.get_buffer_len() && !engine.is_empty() {
                            engine.pop();
                            buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                        } else if insertion_point < engine.get_buffer_len()
                            && insertion_point > 0
                            && !engine.is_empty()
                        {
                            engine.dec_insertion_point();
                            let insertion_point = engine.get_insertion_point();
                            engine.remove_char(insertion_point);
                            buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                        }
                    }
                    KeyCode::Char('w') => {
                        let old_insertion_point = engine.get_insertion_point();
                        engine.move_word_left();
                        if engine.get_insertion_point() < old_insertion_point {
                            cut_buffer.replace_range(
                                ..,
                                &engine.get_buffer()
                                    [engine.get_insertion_point()..old_insertion_point],
                            );
                            engine.clear_range(engine.get_insertion_point()..old_insertion_point);
                            buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                        }
                    }
                    // Ctl left/right or Alt left/right DON"T work!
                    // but they do work on the Linux machine.
                    KeyCode::Left => {
                        engine.run_edit_commands(&[EditCommand::MoveWordLeft]);
                        buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                    }
                    KeyCode::Right => {
                        engine.run_edit_commands(&[EditCommand::MoveWordRight]);
                        buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                    }
                    _ => {}
                },
                Event::Key(KeyEvent {
                    code,
                    modifiers: KeyModifiers::ALT,
                }) => match code {
                    KeyCode::Char('b') => {
                        engine.run_edit_commands(&[EditCommand::MoveLeft]);
                        buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                    }
                    KeyCode::Char('f') => {
                        engine.run_edit_commands(&[EditCommand::MoveRight]);
                        buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                    }
                    KeyCode::Char('d') => {
                        let old_insertion_point = engine.get_insertion_point();
                        engine.move_word_right();
                        if engine.get_insertion_point() > old_insertion_point {
                            cut_buffer.replace_range(
                                ..,
                                &engine.get_buffer()
                                    [old_insertion_point..engine.get_insertion_point()],
                            );
                            engine.clear_range(old_insertion_point..engine.get_insertion_point());
                            engine.set_insertion_point(old_insertion_point);
                            buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                        }
                    }
                    KeyCode::Left => {
                        engine.run_edit_commands(&[EditCommand::MoveWordLeft]);
                        buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                    }
                    KeyCode::Right => {
                        engine.run_edit_commands(&[EditCommand::MoveWordRight]);
                        buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                    }
                    _ => {}
                },
                Event::Key(KeyEvent { code, modifiers: _ }) => {
                    match code {
                        KeyCode::Char(c) => {
                            engine.run_edit_commands(&[
                                EditCommand::InsertChar(c),
                                EditCommand::MoveRight,
                            ]);

                            buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                        }
                        KeyCode::Backspace => {
                            engine.run_edit_commands(&[EditCommand::Backspace]);
                            buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                        }
                        KeyCode::Delete => {
                            engine.run_edit_commands(&[EditCommand::Delete]);
                            buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                        }
                        KeyCode::Home => {
                            engine.run_edit_commands(&[EditCommand::MoveToStart]);
                            buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                        }
                        KeyCode::End => {
                            engine.run_edit_commands(&[EditCommand::MoveToEnd]);
                            buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                        }
                        KeyCode::Enter => {
                            if engine.get_buffer() == "exit" {
                                break 'repl;
                            } else {
                                if history.len() + 1 == HISTORY_SIZE {
                                    // History is "full", so we delete the oldest entry first,
                                    // before adding a new one.
                                    history.pop_back();
                                }
                                history.push_front(String::from(engine.get_buffer()));
                                has_history = true;
                                // reset the history cursor - we want to start at the bottom of the
                                // history again.
                                history_cursor = -1;
                                print_message(
                                    &mut stdout,
                                    &format!("Our engine: {}", engine.get_buffer()),
                                )?;
                                engine.clear();
                                engine.set_insertion_point(0);
                                break 'input;
                            }
                        }
                        KeyCode::Up => {
                            // Up means: navigate through the history.
                            if has_history && history_cursor < (history.len() as i64 - 1) {
                                history_cursor += 1;
                                let history_entry =
                                    history.get(history_cursor as usize).unwrap().clone();
                                engine.set_buffer(history_entry.clone());
                                engine.move_to_end();
                                buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                            }
                        }
                        KeyCode::Down => {
                            // Down means: navigate forward through the history. If we reached the
                            // bottom of the history, we clear the engine, to make it feel like
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

                            engine.set_buffer(new_buffer.clone());
                            engine.move_to_end();
                            buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                        }
                        KeyCode::Left => {
                            engine.run_edit_commands(&[EditCommand::MoveLeft]);
                            buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                        }
                        KeyCode::Right => {
                            engine.run_edit_commands(&[EditCommand::MoveRight]);
                            buffer_repaint(&mut stdout, &engine, prompt_offset)?;
                        }
                        _ => {}
                    };
                }
                Event::Mouse(event) => {
                    print_message(&mut stdout, &format!("{:?}", event))?;
                }
                Event::Resize(width, height) => {
                    print_message(
                        &mut stdout,
                        &format!("width: {} and height: {}", width, height),
                    )?;
                }
            }
        }
    }
    terminal::disable_raw_mode()?;

    println!();
    Ok(())
}
