use crate::Config;
use bevy::{prelude::*, utils::HashMap};
use bevy_ggrs::{LocalInputs, LocalPlayers};
use std::f32::consts::TAU;

const PITCH_UP: u8 = 1 << 0;
const PITCH_DOWN: u8 = 1 << 1;
const YAW_LEFT: u8 = 1 << 2;
const YAW_RIGHT: u8 = 1 << 3;
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
            input |= PITCH_UP;
        }
        if keys.any_pressed([KeyCode::Down, KeyCode::S]) {
            input |= PITCH_DOWN;
        }
        if keys.any_pressed([KeyCode::Left, KeyCode::A]) {
            input |= YAW_LEFT
        }
        if keys.any_pressed([KeyCode::Right, KeyCode::D]) {
            input |= YAW_RIGHT;
        }
        if keys.any_pressed([KeyCode::Space, KeyCode::Return]) {
            input |= FIRE;
        }

        local_inputs.insert(*handle, input);
    }
    commands.insert_resource(LocalInputs::<Config>(local_inputs));
}

pub fn rotation(input: u8) -> Vec3 {
    let delta = TAU / 240.;
    let mut rotation = Vec3::ZERO;
    if input & PITCH_UP != 0 {
        rotation.z += delta;
    }
    if input & PITCH_DOWN != 0 {
        rotation.z -= delta;
    }
    if input & YAW_RIGHT != 0 {
        rotation.y -= delta;
    }
    if input & YAW_LEFT != 0 {
        rotation.y += delta;
    }
    rotation
}

pub fn fire(input: u8) -> bool {
    input & FIRE != 0
}
