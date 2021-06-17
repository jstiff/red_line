## Notes

- there is some default terminal functionality that is built in.
  - echo key events to stdout, for example...
  - there is 'Normal mode and Raw mode'.
    - raw mode will not do much. If there is a key event, it will just take the key value.
  - In normal mode, when you press 'Enter' it will run the carriage return...line feed functionality.
- Episode 2
  - Start by talking about 'separation of concerns'.
    - meaning, we want to take the user input from the prompt and send it off to be processed.
    - build an 'engine' that handles the input commands and sends stuff back to the terminal.
    - build something that can take in commands...apply then to a 'buffer'
- right now we have things combined into one file and even have multiple steps intertwined.
  - for example, will want to break things like 'Backspace' event into two diff blocks of code. One that handles updating the buffer and one that handles updating the screen.
- So, in the end...after episdoe 1. All we are really doing is 'echoing' the input back to stdout.

In the first episode...or first iteration of this we accessed the buffer with a 'slice'...which is bad because it will give you a slice into Bytes. There is the potential to get some bytes missing from the UTF-8 encoding. Rust will be recieving missing byte data to represent that utf-8 character and it will panic!

```rs

stdout
    .queue(Print(c))?
    .queue(Print(&buffer[insertion_point..]))? <-------------------- slice into Bytes !!
    .queue(MoveToColumn(caret_pos + 1))?;


```

We will need to refactor to insure that we are getting a slice at a **character** NOT a byte boundry!!!! - slicing on the Character of a string VS slicing on the Bytes of a string........????
