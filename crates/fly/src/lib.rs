use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct Shell {
    stderr: StandardStream,
    stdout: StandardStream,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            stderr: StandardStream::stderr(ColorChoice::Auto),
            stdout: StandardStream::stdout(ColorChoice::Auto),
        }
    }

    pub fn status(&mut self, status: &str, message: &str) -> io::Result<()> {
        self.stderr.reset()?;
        self.stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;
        write!(self.stderr, "{:>12}", status)?;
        self.stderr.reset()?;
        writeln!(self.stderr, " {}", message)?;

        Ok(())
    }
}
