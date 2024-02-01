use bevy::window::WindowResolution;
use bevy::winit::WinitWindows;
use bevy::{prelude::*, window::WindowCreated};
use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum Side {
    #[default]
    Left,
    Right,
}

#[derive(Parser, Resource, Debug, Clone)]
pub struct Args {
    #[arg(long, value_enum)]
    pub side: Side,
}

impl Plugin for crate::SizedWindowPlugin {
    fn build(&self, app: &mut App) {
        let (width, height) = get_primary_monitor_size().expect("Cannot get monitor size");
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(width as f32 / 2., height as f32 / 2.),
                position: WindowPosition::new(IVec2::new(0, 0)),
                ..default()
            }),
            ..default()
        }));
    }
}

/*
pub fn get_window() -> Window {
    let (resolution, position) = match get_primary_monitor_size() {
        Ok((width, height)) => (
            WindowResolution::new(width as f32 / 2., height as f32 / 2.),
            WindowPosition::new(IVec2::new(0, 0)),
        ),
        Err(_) => {
            error!("Could not get monitor resolution");
            (WindowResolution::default(), WindowPosition::default())
        }
    };
    let fit_canvas_to_parent = false;
    Window {
        resolution,
        position,
        fit_canvas_to_parent,
        ..default()
    }
}
*/

pub fn get_primary_monitor_size() -> Option<(f32, f32)> {
    use winit::event_loop::EventLoop;
    let primary_monitor = EventLoop::new().ok()?.primary_monitor()?;
    let size = primary_monitor.size();
    Some((size.width as f32, size.height as f32))
}
