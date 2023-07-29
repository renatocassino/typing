extern crate termion;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{async_stdin, clear, cursor, style};

use std::collections::VecDeque;

use std::io::{self, Read, Write};
use std::thread;
use std::time::Duration;

struct Game<R, W: Write> {
    text: String,
    stdout: W,
    stdin: R,
}

impl<R: Read, W: Write> Game<R, W> {
    fn new(stdin: R, stdout: W) -> Game<R, RawTerminal<W>> {
        Game {
            text: String::from("Hello, world!"),
            stdin: stdin,
            stdout: stdout.into_raw_mode().unwrap(),
        }
    }

    fn start(&mut self) {
        write!(self.stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();
        self.stdout.write(b"Text").unwrap();
    }
}

fn main() {
    let stdout = io::stdout();
    let mut game = Game::new(async_stdin(), stdout.lock());
    game.start();
}
