use bevy::prelude::*;
use bevy_ggrs::{ggrs::DesyncDetection, prelude::*, *};
use bevy_matchbox::prelude::*;
use bevy_roll_safe::prelude::*;

use clap::Parser;
use components::*;
use input::*;

mod components;
mod input;

type Config = bevy_ggrs::GgrsConfig<u8, PeerId>;

#[derive(Parser, Resource, Debug, Clone)]
pub struct Args {
    /// runs the game in synctest mode
    #[clap(long)]
    pub synctest: bool,
    #[clap(long, default_value = "2")]
    pub input_delay: usize,
}

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum GameState {
    #[default]
    Matchmaking,
    InGame,
}

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum RollbackState {
    #[default]
    InRound,
}

#[derive(Resource, Clone, Deref, DerefMut)]
struct RoundEndTimer(Timer);

impl Default for RoundEndTimer {
    fn default() -> Self {
        RoundEndTimer(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            }),
            GgrsPlugin::<Config>::default(),
        ))
        .add_ggrs_state::<RollbackState>()
        .rollback_resource_with_clone::<RoundEndTimer>()
        .rollback_component_with_clone::<Transform>()
        .rollback_component_with_copy::<BulletReady>()
        .rollback_component_with_copy::<Player>()
        .rollback_component_with_copy::<MoveDir>()
        .rollback_component_with_clone::<GlobalTransform>()
        .rollback_component_with_clone::<Visibility>()
        .rollback_component_with_clone::<InheritedVisibility>()
        .rollback_component_with_clone::<ViewVisibility>()
        .checksum_component::<Transform>(checksum_transform)
        .insert_resource(ClearColor(Color::rgb(0.53, 0.53, 0.53)))
        .init_resource::<RoundEndTimer>()
        .add_systems(
            OnEnter(GameState::Matchmaking),
            (setup, start_matchbox_socket),
        )
        .add_systems(
            Update,
            (
                wait_for_players.run_if(in_state(GameState::Matchmaking)),
                (handle_ggrs_events).run_if(in_state(GameState::InGame)),
            ),
        )
        .add_systems(ReadInputs, read_local_inputs)
        .add_systems(OnEnter(RollbackState::InRound), spawn_players)
        .add_systems(
            GgrsSchedule,
            (
                move_players,
                reload_bullet,
                fire_bullets.after(move_players).after(reload_bullet),
                move_bullet.after(fire_bullets),
            )
                .run_if(in_state(RollbackState::InRound))
                .after(apply_state_transition::<RollbackState>),
        )
        .run();
}

const MAP_SIZE: i32 = 41;

fn setup(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-0.5, -0.5, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

use std::f32::consts::TAU;

fn spawn_players(
    mut commands: Commands,
    players: Query<Entity, With<Player>>,
    bullets: Query<Entity, With<Bullet>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for player in &players {
        commands.entity(player).despawn_recursive();
    }

    for bullet in &bullets {
        commands.entity(bullet).despawn_recursive();
    }

    commands
        .spawn((
            Player { handle: 0 },
            BulletReady(true),
            MoveDir(-Vec3::X),
            PbrBundle {
                mesh: meshes.add(
                    Mesh::try_from(shape::Cube {
                        size: 1.0,
                        ..Default::default()
                    })
                    .unwrap(),
                ),
                material: materials.add(Color::GREEN.into()),
                transform: Transform::from_translation(Vec3::new(-2.0, 0.0, 0.0))
                    .with_rotation(Quat::from_rotation_y(TAU / -4.0)),
                ..Default::default()
            },
        ))
        .with_children(|child| {
            child.spawn(PbrBundle {
                mesh: meshes.add(
                    Mesh::try_from(shape::Capsule {
                        radius: 0.05,
                        depth: 1.0,
                        ..Default::default()
                    })
                    .unwrap(),
                ),
                material: materials.add(Color::WHITE.into()),
                transform: Transform::from_rotation(Quat::from_rotation_x(TAU / 4.0))
                    .with_translation(Vec3::Z * -1.0),
                ..Default::default()
            });
        })
        .add_rollback();
    commands
        .spawn((
            Player { handle: 1 },
            BulletReady(true),
            MoveDir(-Vec3::X),
            PbrBundle {
                mesh: meshes.add(
                    Mesh::try_from(shape::Cube {
                        size: 1.0,
                        ..Default::default()
                    })
                    .unwrap(),
                ),
                material: materials.add(Color::BLUE.into()),
                transform: Transform::from_translation(Vec3::new(2.0, 0.0, 0.0))
                    .with_rotation(Quat::from_rotation_y(TAU / 4.0)),
                ..Default::default()
            },
        ))
        .with_children(|child| {
            child.spawn(PbrBundle {
                mesh: meshes.add(
                    Mesh::try_from(shape::Capsule {
                        radius: 0.05,
                        depth: 1.0,
                        ..Default::default()
                    })
                    .unwrap(),
                ),
                material: materials.add(Color::WHITE.into()),
                transform: Transform::from_rotation(Quat::from_rotation_x(TAU / 4.0))
                    .with_translation(Vec3::Z * -1.0),
                ..Default::default()
            });
        })
        .add_rollback();
}

fn start_matchbox_socket(mut commands: Commands) {
    let room_url = "ws://127.0.0.1:3536/extreme_bevy?next=2";
    commands.insert_resource(MatchboxSocket::new_ggrs(room_url));
    info!("started matchbox socket");
}

fn wait_for_players(
    mut commands: Commands,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if socket.get_channel(0).is_err() {
        return;
    }

    socket.update_peers();
    let players = socket.players();

    let num_players = 2;
    if players.len() < num_players {
        return;
    }

    let mut session_builder = ggrs::SessionBuilder::<Config>::new()
        .with_num_players(num_players)
        .with_desync_detection_mode(DesyncDetection::On { interval: 1 })
        .with_input_delay(2);

    for (i, player) in players.into_iter().enumerate() {
        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");
    }

    let socket = socket.take_channel(0).unwrap();

    let ggrs_session = session_builder
        .start_p2p_session(socket)
        .expect("failed to start session");

    commands.insert_resource(bevy_ggrs::Session::P2P(ggrs_session));
    next_state.set(GameState::InGame);
}

fn handle_ggrs_events(mut session: ResMut<Session<Config>>) {
    match session.as_mut() {
        Session::P2P(s) => {
            for event in s.events() {
                match event {
                    GgrsEvent::Disconnected { .. } | GgrsEvent::NetworkInterrupted { .. } => {
                        error!("disconnected");
                    }
                    GgrsEvent::DesyncDetected { .. } => {
                        error!("desynced");
                    }
                    _ => {
                        error!("unexpected event: {event:?}");
                    }
                }
            }
        }
        _ => {}
    }
}

fn move_players(
    mut players: Query<(&mut Transform, &mut MoveDir, &Player)>,
    inputs: Res<PlayerInputs<Config>>,
    time: Res<Time>,
) {
    for (mut transform, mut move_direction, player) in &mut players {
        let (input, _) = inputs[player.handle];

        let direction = direction(input);

        if direction == Vec3::ZERO {
            continue;
        }

        move_direction.0 = direction;

        let move_speed = 7.;
        let move_delta = direction * move_speed * time.delta_seconds();

        let old_pos = transform.translation;
        let limit = Vec3::splat(MAP_SIZE as f32 / 2. - 0.5);
        let new_pos = (old_pos + move_delta).clamp(-limit, limit);

        transform.translation.x = new_pos.x;
        transform.translation.y = new_pos.y;
    }
}

fn reload_bullet(
    inputs: Res<PlayerInputs<Config>>,
    mut players: Query<(&mut BulletReady, &Player)>,
) {
    for (mut can_fire, player) in players.iter_mut() {
        let (input, _) = inputs[player.handle];
        if !fire(input) {
            can_fire.0 = true;
        }
    }
}

fn fire_bullets(
    mut commands: Commands,
    inputs: Res<PlayerInputs<Config>>,
    mut players: Query<(&Transform, &Player, &mut BulletReady)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (&transform, player, mut bullet_ready) in &mut players {
        let (input, _) = inputs[player.handle];
        if fire(input) && bullet_ready.0 {
            let player_pos = transform.translation;
            let forward = -transform.local_z();
            let pos = player_pos + forward * PLAYER_RADIUS + BULLET_RADIUS;
            commands
                .spawn((
                    Bullet,
                    MoveDir(forward),
                    PbrBundle {
                        mesh: meshes.add(
                            Mesh::try_from(shape::Icosphere {
                                radius: 0.04,
                                ..Default::default()
                            })
                            .unwrap(),
                        ),
                        material: materials.add(Color::RED.into()),
                        transform: Transform::from_translation(pos),
                        ..Default::default()
                    },
                ))
                .add_rollback();
            bullet_ready.0 = false;
        }
    }
}

fn move_bullet(mut bullets: Query<(&mut Transform, &MoveDir), With<Bullet>>, time: Res<Time>) {
    for (mut transform, dir) in &mut bullets {
        let speed = 20.;
        let delta = dir.0 * speed * time.delta_seconds();
        transform.translation += delta;
    }
}

const PLAYER_RADIUS: f32 = 0.5;
const BULLET_RADIUS: f32 = 0.05;
