use bevy::prelude::*;

pub trait WasmAppExtensions {
    fn extend_wasm(&mut self) -> &mut Self;
}

impl WasmAppExtensions for App {
    fn extend_wasm(&mut self) -> &mut Self {
        self.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
    }
}
