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
- So, in the end...after episode 1. All we are really doing is 'echoing' the input back to stdout.

In the first episode...or first iteration of this, we access the buffer with a 'slice'...which is bad because it will give you a slice into Bytes. There is the potential to get some bytes missing from the UTF-8 encoding. Rust will be receiving missing byte data to represent that utf-8 character, and it will panic!

```rs

stdout
    .queue(Print(c))?
    .queue(Print(&buffer[insertion_point..]))? <-------------------- slice into Bytes !!
    .queue(MoveToColumn(caret_pos + 1))?;


```

We will need to refactor to ensure that we are getting a slice of a **character,** NOT a byte boundary!!!! - slicing on the Character of a string VS slicing on the Bytes of a string........???
```
A Rust String is a vector of bytes containing a UTF-8 string, which is an uneasy combination. You can’t simply index into a String: the compiler forbids it, because you’re far too likely to get one byte of a multi-byte UTF-8 char. Instead, you need to use a Chars iterator to parse out the string character by character.

---

### Part 3

- moving the buffer updating at the beginning of the 'KeyCode' events....
- Might want to optimize the 'insert_char' method later if that's possible. Right now it copies everything over in memory.

- Going to implement the 'Command pattern' for these key events. So that the interface simply displays the 'key' and it's subsequent function calls and that's it. Hide everything else. I am assuming that we will create an 'engine' module that handles the logic for everything.

- want the engine to handle the logic and the 'main.rs' function to be simplified. Inside the first iteration of the 'engine' we will have a strut called 'engine' that will hold the 'LineBuffer'...and for the time being when we call for methods on the LineBuffer we will have the Engine stuct simply proxy for the line_buffer logic we already have. This is just an indirection tactic....we will have the interface be the engine...then the engine redirects to call the LineBuffer logic that we already set up. 
- Eventually we will start peeling away at that. 
  -  We just copy and pasted LineBuffer impl logic into Engine...will refactor in order to redirect into the 'line_buffer'...this may cause issues with the compiler,  because we could run into scenarios where we have a reference to a reference, and it will complain. 

- 

