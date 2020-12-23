use core::panic::PanicInfo;
use std::{
    env,
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
    process, writeln,
};

use indoc::indoc;
use preflight_impl::{Avionics, Sensors};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use uuid::Uuid;

use crate::args::PanicHandleArguments;

pub fn panic_handle(
    panic_info: &PanicInfo,
    avionics: &dyn Avionics,
    sensors: &Sensors,
    args: &PanicHandleArguments,
) {
    let file_path = panic_file();

    {
        let mut file = File::create(&file_path).expect("Failed to create panic report file");

        write!(
            file,
            indoc! {"
                {:#?}
                
                //----INPUT----
                {:#?}
                
                //----CURRENT STATE----
                {:#?}
            "},
            panic_info, sensors, avionics
        )
        .expect("Failed to write to the panic report");
    }

    panic_alert(panic_info, &file_path).expect("Failed to warn the user of the panic");

    if args.open {
        open::that(file_path).expect("Failed to open the panic report");
    }

    process::exit(1);
}

pub fn panic_file() -> PathBuf {
    let uuid = Uuid::new_v4().to_hyphenated().to_string();
    let tmp_dir = env::temp_dir();
    let file_name = format!("{}.panic.rs", &uuid);
    Path::new(&tmp_dir).join(file_name)
}

pub fn panic_alert(panic_info: &PanicInfo, file: &Path) -> io::Result<()> {
    let mut stderr = StandardStream::stderr(ColorChoice::Auto);

    stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
    writeln!(stderr, "\nGUIDANCE SYSTEM PANIC!")?;

    stderr.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
    writeln!(stderr, "{}", panic_info)?;

    stderr.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
    write!(
        stderr,
        indoc! {"

            In flight this would trigger an auto abort, but should be avoided at all costs.
            Detailed information can be found at "
        }
    )?;

    stderr.set_color(
        ColorSpec::new()
            .set_intense(true)
            .set_fg(Some(Color::Magenta)),
    )?;
    writeln!(stderr, "{}", file.to_string_lossy())
}
