use conrod::color::Color;
use kiss3d::{
    conrod::{
        self,
        position::Padding,
        position::{Align, Direction, Relative},
        widget,
        widget::Canvas,
        widget::PlotPath,
        Colorable, Labelable, Position, Positionable, Sizeable, Theme, Widget,
    },
    widget_ids,
};
use log::trace;
use timescale::InterpolatedDataTable;

use crate::motors::EstesC6;

pub fn theme() -> conrod::Theme {
    Theme {
        name: "Demo Theme".to_string(),
        padding: Padding::none(),
        x_position: Position::Relative(Relative::Align(Align::Start), None),
        y_position: Position::Relative(Relative::Direction(Direction::Backwards, 20.0), None),
        background_color: conrod::color::DARK_CHARCOAL,
        shape_color: conrod::color::LIGHT_CHARCOAL,
        border_color: conrod::color::BLACK,
        border_width: 0.0,
        label_color: conrod::color::WHITE,
        font_id: None,
        font_size_large: 26,
        font_size_medium: 18,
        font_size_small: 12,
        widget_styling: conrod::theme::StyleMap::default(),
        mouse_drag_threshold: 0.0,
        double_click_threshold: std::time::Duration::from_millis(500),
    }
}

widget_ids! {
    pub struct Ids {
        // The scrollable canvas.
        canvas,
        // The title and introduction widgets.
        title_vel_x_slider,
        vel_x_slider,
        // The widget used for graphs
        motor_graph
    }
}

pub fn gui(ui: &mut conrod::UiCell, ids: &Ids) {
    const SIDEBAR_W: f64 = 200.0;
    const ELEMENT_W: f64 = SIDEBAR_W - 20.0;
    const ELEMENT_H: f64 = 20.0;
    const VSPACE: f64 = 4.0;
    const TITLE_VSPACE: f64 = 4.0;
    const LEFT_MARGIN: f64 = 10.0;
    const ALPHA: f32 = 0.9;

    Canvas::new()
        .title_bar("Demos")
        .title_bar_color(Color::Rgba(1.0, 0.0, 0.0, 1.0))
        //            .pad(100.0)
        //            .pad_left(MARGIN)
        //            .pad_right(MARGIN)
        .scroll_kids_vertically()
        .mid_right_with_margin(10.0)
        .w(SIDEBAR_W)
        .padded_h_of(ui.window, 10.0)
        .set(ids.canvas, ui);

    PlotPath::new(0.0, 10.0, 0.0, 1000.0, |x| EstesC6::get(x).thrust)
        // .title("Estes C6 thrust (N)")
        .set(ids.motor_graph, ui);

    // conrod::widget::Text::new("Vel. Iters.:")
    //     .set(ids.title_vel_x_slider, &mut ui);

    // for val in conrod::widget::Slider::new(curr_vel_iters as f32, 0.0, 50.0)
    //     .label(&curr_vel_iters.to_string())
    //     .align_middle_x_of(ids.canvas)
    //     .down_from(ids.title_slider_vel_iter, TITLE_VSPACE)
    //     .w_h(ELEMENT_W, ELEMENT_H)
    //     .set(ids.slider_vel_iter, &mut ui)
    // {
    //     world.integration_parameters.max_velocity_iterations = val as usize;
    // }
}
