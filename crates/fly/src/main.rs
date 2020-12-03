use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use std::io::Write;

fn main() {
    let mut output = StandardStream::stderr(ColorChoice::Auto);
    output.reset().unwrap();
    output
        .set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))
        .unwrap();
    write!(output, "{:>12} ", "Doin").unwrap();
    output.reset().unwrap();
    writeln!(output, "your mom").unwrap();
}
