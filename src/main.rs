use crossterm::event::KeyEvent;
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
    current_sequence: u32,
    bigger_sequence: u32,
}

impl Score {
    fn new() -> Score {
        Score {
            start_time: Instant::now(),
            assertions: 0,
            wrong: 0,
            current_sequence: 0,
            bigger_sequence: 0,
        }
    }

    fn wrong(&mut self) {
        self.wrong += 1;
    }

    fn right(&mut self) {
        self.assertions += 1;
        self.current_sequence += 1;
        self.bigger_sequence = self.bigger_sequence.max(self.current_sequence);
    }
}

struct GameConfig {
    camel_case: bool,
}

struct Game {
    stdout: io::Stdout,
    stdin: io::Stdin,
    text: String,
    index: usize,
    score: Score,
    audio: Audio,
    config: GameConfig,
}

impl Game {
    fn new(stdin: io::Stdin, stdout: io::Stdout, config: GameConfig) -> Game {
        Game {
            stdout,
            stdin,
            text: String::from(""),
            index: 0,
            score: Score::new(),
            audio: Audio::new(),
            config,
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

    fn is_correct_char_pressed(&mut self, event: KeyEvent) -> bool {
        let current_char = self.text.chars().nth(self.index).unwrap();

        if event.code == crossterm::event::KeyCode::Char(current_char) {
            return true;
        }
        if (self.config.camel_case) {
            return false;
        }

        return event.code == crossterm::event::KeyCode::Char(current_char.to_ascii_lowercase())
            || event.code == crossterm::event::KeyCode::Char(current_char.to_ascii_uppercase());
    }

    fn print_events(&mut self) -> crossterm::Result<()> {
        self.print_text()?;
        loop {
            match read()? {
                Event::Key(event) => {
                    if event.code == crossterm::event::KeyCode::Esc {
                        break;
                    }

                    if self.is_correct_char_pressed(event) {
                        self.index += 1;
                        self.score.right();
                        self.audio.play("press");
                    } else {
                        self.score.wrong();
                        self.audio.play("wrong");
                    }

                    self.print_text()?;

                    if self.index == self.text.len() {
                        break;
                    }
                }
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
            if i < self.index {
                queue!(
                    self.stdout,
                    cursor::MoveTo(x, y),
                    style::PrintStyledContent(c.green())
                )?;
            } else if i == self.index {
                queue!(
                    self.stdout,
                    cursor::MoveTo(x, y),
                    SetBackgroundColor(Color::DarkYellow),
                    style::PrintStyledContent(c.blue()),
                    SetBackgroundColor(Color::Black),
                )?;
            } else {
                queue!(
                    self.stdout,
                    cursor::MoveTo(x, y),
                    style::PrintStyledContent(c.red())
                )?;
            }

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

    fn final_score(&mut self) -> Result<()> {
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
        queue!(
            self.stdout,
            cursor::MoveTo(5, 11),
            style::PrintStyledContent(
                format!("Bigger sequence: {}", self.score.bigger_sequence).green()
            )
        )?;
        queue!(
            self.stdout,
            cursor::MoveTo(5, 12),
            style::PrintStyledContent("\n\n\n".green())
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
    let mut game = Game::new(stdin, stdout, GameConfig { camel_case: false });
    game.start()?;
    game.final_score()?;
    disable_raw_mode()?;
    Ok(())
}
