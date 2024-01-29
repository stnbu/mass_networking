use crate::GameState;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;
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

pub trait Aarch64AppExtensions {
    fn extend_aarch64(&mut self) -> &mut Self;
}

impl Aarch64AppExtensions for App {
    fn extend_aarch64(&mut self) -> &mut Self {
        let args = Args::parse();
        self.add_plugins(DefaultPlugins)
            .insert_resource(args)
            .add_systems(OnEnter(GameState::InGame), position_window)
    }
}

pub fn position_window(
    args: Res<Args>,
    mut windows: Query<&mut Window>,
    winit_windows: NonSend<WinitWindows>,
    window_query: Query<Entity, With<PrimaryWindow>>,
) {
    let display_size = if let Some(monitor) = window_query
        .get_single()
        .ok()
        .and_then(|entity| winit_windows.get_window(entity))
        .and_then(|winit_window| winit_window.current_monitor())
    {
        monitor.size()
    } else {
        panic!("No monitor found!");
    };

    let display_width = display_size.width;
    let display_height = display_size.height;

    let window_width = display_width / 4.0 as u32;
    let window_height = display_height / 4.0 as u32;
    let window_x = match args.side {
        Side::Left => 0,
        Side::Right => display_width - (window_width * 2),
    };

    let mut window = windows.single_mut();
    window
        .resolution
        .set(window_width as f32, window_height as f32);
    window.position.set(IVec2::new(window_x as i32, 0.0 as i32));
}
