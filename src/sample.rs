use nannou::prelude::*;

/// Entry point for this sample application.
///
/// Initializes the nannou app with the `model`, `update`, and `view`
/// functions and starts the main event loop.
pub fn run() {
    nannou::app(model).update(update).simple_window(view).run();
}

struct Model {
    points: Vec<Point2>,
}

fn model(app: &App) -> Model {
    app.set_loop_mode(LoopMode::RefreshSync);
    Model { points: Vec::new() }
}

fn update(app: &App, model: &mut Model, _: Update) {
    let t = app.time;
    let r = 200.0 + 50.0 * (t * 0.5).sin();
    let angle = t * 0.7;
    let x = r * angle.cos();
    let y = r * angle.sin();
    model.points.push(pt2(x, y));
    if model.points.len() > 800 {
        model.points.remove(0);
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);
    // let alpha = 0.6;
    for (i, p) in model.points.iter().enumerate() {
        let w = 2.0 + (i as f32 * 0.01);
        let hue = (i as f32 / model.points.len().max(1) as f32) * 0.8;
        draw.ellipse().xy(*p).radius(w).hsl(hue, 0.6, 0.5);
        // .alpha(alpha);
    }
    draw.to_frame(app, &frame).unwrap();
}
