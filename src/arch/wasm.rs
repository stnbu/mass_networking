use bevy::prelude::*;

pub trait ArchAppExt {
    fn arch_build(&mut self) -> &mut Self;
}

impl ArchAppExt for App {
    fn arch_build(&mut self) -> &mut Self {
        self.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
    }
}
