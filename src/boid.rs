use bevy::ecs::system::Command;
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct SpawnBoid;

impl Command for SpawnBoid {
    fn apply(self, world: &mut World) {
        world.spawn(SpawnBoid::default());
    }
}

pub fn boid(mut commands: Commands) {
    for _ in 0..100 {
        warn!("WARNING!! spawning the boid...");
        commands.add(SpawnBoid::default());
    }
}
