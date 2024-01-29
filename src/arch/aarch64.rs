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

pub trait WasmAppExtensions {
    fn extend_aarch64(&mut self) -> &mut Self;
}

impl WasmAppExtensions for App {
    fn extend_aarch64(&mut self) -> &mut Self {
        let args = Args::parse();
        self.add_plugins(DefaultPlugins)
            .insert_resource(args)
            // FIXME: Called in every frame; system checks WindowCreated event queue length.
            .add_systems(Update, position_window)
    }
}

pub fn position_window(
    args: Res<Args>,
    mut windows: Query<&mut Window>,
    winit_windows: NonSend<WinitWindows>,
    mut window_created_events: EventReader<WindowCreated>,
) {
    if window_created_events.len() == 0 {
        return;
    }
    let &WindowCreated { window: monitor } = window_created_events.read().next().unwrap();
    let display_size = winit_windows
        .get_window(monitor)
        .expect("Could not get WinitWindow")
        .current_monitor()
        .expect("Could not get current monitor")
        .size();

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
