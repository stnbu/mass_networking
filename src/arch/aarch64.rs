use crate::GameState;
use bevy::log::LogPlugin;
use bevy::winit::WinitWindows;
use bevy::{prelude::*, window::WindowCreated};
use bevy_ggrs::LocalPlayers;
use clap::{Parser, ValueEnum};
use tracing_appender::rolling;

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

pub trait ArchAppExt {
    fn arch_build(&mut self) -> &mut Self;
}

impl ArchAppExt for App {
    fn arch_build(&mut self) -> &mut Self {
        let args = Args::parse();

        self.add_plugins(DefaultPlugins.build().disable::<LogPlugin>())
            .insert_resource(args)
            .add_systems(OnEnter(GameState::InGame), setup_file_logging)
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

fn setup_file_logging(local_players: Res<LocalPlayers>) {
    assert!(local_players.0.len() == 1);
    for &local_player in &local_players.0 {
        let file_appender = rolling::never(".", format!("handle_{}.log", local_player));
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        tracing_subscriber::fmt()
            .with_writer(non_blocking)
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }
}
