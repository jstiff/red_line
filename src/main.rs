use crossterm::{
    event::{poll, read, Event, KeyCode},
    terminal, Result,
};

use std::io::stdout;
use std::io::Stdout;
use std::time::Duration;
mod engine;
use engine::{print_message, Engine};
mod line_buffer;

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

    loop {
        if let Ok(buffer) = engine.read_line(&mut stdout) {
            if buffer.trim() == "exit" {
                break;
            }
            if !buffer.trim().is_empty() {
                print_message(&mut stdout, &format!("Our Buffer: {}", buffer))?
            }
        }
    }
    terminal::disable_raw_mode()?;

    println!();
    Ok(())
}
