use ggez::ContextBuilder;
use std::path::PathBuf;
use std::env;
use ggez::conf::WindowMode;
use ggez::conf::WindowSetup;
use ggez::conf::NumSamples;
use ggez::conf::FullscreenType;

use crate::consts::SCREEN_SIZE;

pub fn build_window() -> ContextBuilder {

    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        PathBuf::from("./resources")
    };

    let cb: ContextBuilder = ggez::ContextBuilder::new("Snake in Rust", "Bartosz Jaśkiewicz")
        .add_resource_path(resource_dir)
        .window_setup(WindowSetup::default().title("Snake in Rust - project")
                                            .samples(NumSamples::Zero)
                                            .vsync(true))
        .window_mode(WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1)
                                          .fullscreen_type(FullscreenType::Windowed)
                                          .resizable(true));
    cb
}