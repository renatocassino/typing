use crossterm::style::{Color, SetBackgroundColor, Stylize};
use crossterm::{
    cursor,
    event::{read, Event},
    execute, queue,
    style::{self},
    terminal,
    terminal::{disable_raw_mode, enable_raw_mode},
    Result,
};
use rusty_audio::Audio;
use std::io::{self, stdout, Write};
use std::time::Instant;

struct Score {
    start_time: Instant,
    assertions: u32,
    wrong: u32,
}

impl Score {
    fn new() -> Score {
        Score {
            start_time: Instant::now(),
            assertions: 0,
            wrong: 0,
        }
    }

    fn wrong(&mut self) {
        self.wrong += 1;
    }

    fn right(&mut self) {
        self.assertions += 1;
    }
}

struct Game {
    stdout: io::Stdout,
    stdin: io::Stdin,
    text: String,
    index: usize,
    score: Score,
    audio: Audio,
}

impl Game {
    fn new(stdin: io::Stdin, stdout: io::Stdout) -> Game {
        Game {
            stdout: stdout,
            stdin: stdin,
            text: String::from(""),
            index: 0,
            score: Score::new(),
            audio: Audio::new(),
        }
    }

    fn build_dash(&mut self) -> Result<()> {
        execute!(self.stdout, terminal::Clear(terminal::ClearType::All))?;

        for y in 0..40 {
            for x in 0..150 {
                if (y == 0 || y == 40 - 1) || (x == 0 || x == 150 - 1) {
                    queue!(
                        self.stdout,
                        cursor::MoveTo(x, y),
                        style::PrintStyledContent("█".magenta())
                    )?;
                }
            }
        }
        self.stdout.flush()?;
        Ok(())
    }

    fn print_events(&mut self) -> crossterm::Result<()> {
        loop {
            self.print_text()?;
            match read()? {
                Event::Key(event) => {
                    if event.code == crossterm::event::KeyCode::Char('q') {
                        break;
                    }

                    if event.code
                        == crossterm::event::KeyCode::Char(
                            self.text.chars().nth(self.index).unwrap(),
                        )
                    {
                        self.index += 1;
                        self.score.right();
                        self.audio.play("press");
                    } else {
                        self.score.wrong();
                        self.audio.play("wrong");
                    }

                    if self.index == self.text.len() {
                        break;
                    }
                }
                Event::Mouse(event) => println!("{:?}", event),
                Event::Resize(width, height) => println!("New size {}x{}", width, height),
                _ => {}
            }
        }
        Ok(())
    }

    fn print_text(&mut self) -> Result<()> {
        let mut x = 5;
        let mut y = 5;

        let mut i = 0;
        for c in self.text.chars() {
            queue!(self.stdout, SetBackgroundColor(Color::Black))?;
            let curr_style = if i < self.index {
                style::PrintStyledContent(c.green())
            } else if i == self.index {
                queue!(self.stdout, SetBackgroundColor(Color::DarkYellow))?;
                style::PrintStyledContent(c.blue())
            } else {
                queue!(self.stdout, SetBackgroundColor(Color::Black))?;
                style::PrintStyledContent(c.red())
            };

            queue!(self.stdout, cursor::MoveTo(x, y), curr_style)?;
            x += 1;
            if x > 130 {
                x = 5;
                y += 1;
            }
            i += 1;
        }

        self.stdout.flush()?;

        Ok(())
    }

    fn load(&mut self) -> Result<()> {
        let text = std::fs::read_to_string("text.txt")?;
        self.text = text;
        self.audio.add("press", "audio/press.mp3");
        self.audio.add("wrong", "audio/wrong.mp3");

        Ok(())
    }

    fn score(&mut self) -> Result<()> {
        let end_time = Instant::now();
        let duration = end_time.duration_since(self.score.start_time);
        let per_seconds = (self.score.assertions as f64) / duration.as_secs_f64();

        execute!(self.stdout, terminal::Clear(terminal::ClearType::All))?;

        queue!(
            self.stdout,
            cursor::MoveTo(5, 5),
            style::PrintStyledContent("Score".green())
        )?;
        queue!(
            self.stdout,
            cursor::MoveTo(5, 6),
            style::PrintStyledContent(format!("Assertions: {}", self.score.assertions).green())
        )?;
        queue!(
            self.stdout,
            cursor::MoveTo(5, 7),
            style::PrintStyledContent(format!("Wrong: {}", self.score.wrong).red())
        )?;
        let total_attempts = self.score.assertions + self.score.wrong;
        queue!(
            self.stdout,
            cursor::MoveTo(5, 8),
            style::PrintStyledContent(
                format!(
                    "Accuracy: {}%",
                    (self.score.assertions as f32 / total_attempts as f32) * 100.0
                )
                .as_str()
                .green()
            )
        )?;
        queue!(
            self.stdout,
            cursor::MoveTo(5, 9),
            style::PrintStyledContent(format!("Per seconds: {:.2}", per_seconds).green())
        )?;
        queue!(
            self.stdout,
            cursor::MoveTo(5, 10),
            style::PrintStyledContent(format!("Per minute: {:.2}", per_seconds * 60.).green())
        )?;
        self.stdout.flush()?;
        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        self.load()?;
        self.build_dash()?;
        self.print_events()?;
        Ok(())
    }
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let stdout = io::stdout();
    let stdin = io::stdin();
    let mut game = Game::new(stdin, stdout);
    game.start()?;
    game.score()?;
    disable_raw_mode()?;
    Ok(())
}
