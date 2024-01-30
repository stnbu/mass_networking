use crate::Config;
use bevy::{prelude::*, utils::HashMap};
use bevy_ggrs::{LocalInputs, LocalPlayers};

const ROTATE_UP: u8 = 1 << 0;
const ROTATE_DOWN: u8 = 1 << 1;
const ROTATE_LEFT: u8 = 1 << 2;
const ROTATE_RIGHT: u8 = 1 << 3;
const FIRE: u8 = 1 << 4;

pub fn read_local_inputs(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    local_players: Res<LocalPlayers>,
) {
    let mut local_inputs = HashMap::new();

    for handle in &local_players.0 {
        let mut input = 0u8;

        if keys.any_pressed([KeyCode::Up, KeyCode::W]) {
            input |= ROTATE_UP;
        }
        if keys.any_pressed([KeyCode::Down, KeyCode::S]) {
            input |= ROTATE_DOWN;
        }
        if keys.any_pressed([KeyCode::Left, KeyCode::A]) {
            input |= ROTATE_LEFT
        }
        if keys.any_pressed([KeyCode::Right, KeyCode::D]) {
            input |= ROTATE_RIGHT;
        }
        if keys.any_pressed([KeyCode::Space, KeyCode::Return]) {
            input |= FIRE;
        }

        local_inputs.insert(*handle, input);
    }
    commands.insert_resource(LocalInputs::<Config>(local_inputs));
}

pub fn _direction(input: u8) -> Vec3 {
    let mut direction = Vec3::ZERO;
    if input & ROTATE_UP != 0 {
        direction.y += 1.;
    }
    if input & ROTATE_DOWN != 0 {
        direction.y -= 1.;
    }
    if input & ROTATE_RIGHT != 0 {
        direction.x += 1.;
    }
    if input & ROTATE_LEFT != 0 {
        direction.x -= 1.;
    }
    direction.normalize_or_zero()
}

pub fn fire(input: u8) -> bool {
    input & FIRE != 0
}
