use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct Shell {
    stderr: StandardStream,
    // stdout: StandardStream,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            stderr: StandardStream::stderr(ColorChoice::Auto),
            // stdout: StandardStream::stdout(ColorChoice::Auto),
        }
    }

    pub fn status<S, M>(&mut self, status: S, message: M) -> io::Result<()>
    where
        S: AsRef<str>,
        M: AsRef<str>,
    {
        self.stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;
        write!(self.stderr, "{:>12}", status.as_ref())?;

        self.stderr.reset()?;
        writeln!(self.stderr, " {}", message.as_ref())
    }

    pub fn error<M>(&mut self, message: M) -> io::Result<()>
    where
        M: AsRef<str>,
    {
        self.stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
        write!(self.stderr, "error")?;

        self.stderr.reset()?;
        writeln!(self.stderr, ": {}", message.as_ref().trim_end())
    }

    pub fn warning<M>(&mut self, message: M) -> io::Result<()>
    where
        M: AsRef<str>,
    {
        self.stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true))?;
        write!(self.stderr, "warning")?;

        self.stderr.set_color(ColorSpec::new().set_bold(true))?;
        writeln!(self.stderr, ": {}", message.as_ref().trim_end())
    }

    pub fn note<M>(&mut self, message: M) -> io::Result<()>
    where
        M: AsRef<str>,
    {
        self.stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Blue)).set_bold(true))?;
        write!(self.stderr, "   = ")?;

        self.stderr.set_color(ColorSpec::new().set_bold(true))?;
        write!(self.stderr, "note:")?;

        self.stderr.reset()?;
        writeln!(self.stderr, " {}", message.as_ref().trim_end())
    }
}
