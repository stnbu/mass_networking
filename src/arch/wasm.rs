use bevy::prelude::*;

impl Plugin for crate::SizedWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }));
    }
}

pub fn get_primary_monitor_size() -> Option<(f32, f32)> {
    None
}
