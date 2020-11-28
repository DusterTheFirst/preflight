use cairo::{FontSlant, FontWeight};
use color_eyre::{eyre::eyre, Help};
use gtk::{
    prelude::BuilderExtManual, ApplicationWindow, Builder, Button, ButtonExt, DialogExt,
    DrawingArea, Inhibit, MessageDialog, WidgetExt, Window,
};
use plotters::{
    prelude::{ChartBuilder, IntoDrawingArea, SurfaceSeries},
    style::{RED, WHITE},
};
use plotters_cairo::CairoBackend;

pub mod simulation;

const ROXIDE_GLADE: &str = include_str!("../glade/roxide.glade");

/// Little macro to help get an object from a builder and propagate the error
macro_rules! get_objects {
    (
        use $builder:ident;
        $(let $name:ident: $ty:ty;)*
    ) => {
        $(let $name: $ty = $builder
            .get_object(stringify!($name))
            .ok_or(eyre!(concat!(
                "Object with id `",
                stringify!($name),
                "` missing from the glade builder"
            )))
            .suggestion("Check the spelling at the location above and in the glade file")?;
        )*
    };
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    gtk::init()
        .note("Failed to initialize GTK")
        .suggestion("Ensure that GTK is installed and v3.24 or higher")?;

    let builder = Builder::from_string(ROXIDE_GLADE);

    get_objects! {
        use builder;
        let application_window: ApplicationWindow;
        let graph_window: Window;
        let motor_graph_button: Button;
    }

    // let application_window: ApplicationWindow = get_object!(builder["application_window"]);
    // let graph_window: Window = get_object!(builder["graph_window"]);

    // let motor_graph_button: Button = get_object!(builder["motor_graph_button"]);
    motor_graph_button.connect_clicked(move |_| graph_window.show_all());

    // let button: Button = get_object!(builder["useless_button"]);
    // let dialog: MessageDialog = get_object!(builder["button_alert"]);

    // let draw: DrawingArea = get_object!(builder["draw"]);

    // draw.connect_draw(|_, cr| {
    //     let root = CairoBackend::new(cr, (500, 500))
    //         .unwrap()
    //         .into_drawing_area();

    //     root.fill(&WHITE).unwrap();

    //     let mut chart = ChartBuilder::on(&root)
    //         .caption("This is a test", ("sans-serif", 20))
    //         .x_label_area_size(40)
    //         .y_label_area_size(40)
    //         .build_cartesian_2d(0..100, 0..100)
    //         .unwrap();

    //     chart.configure_mesh().draw().unwrap();

    //     Inhibit(false)
    // });

    // button.connect_clicked(move |_| {
    //     dialog.run();
    //     dialog.hide();
    // });

    application_window.show_all();

    application_window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    gtk::main();

    Ok(())
}
