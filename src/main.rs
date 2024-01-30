use bevy::prelude::*;
use bevy_ggrs::{ggrs::DesyncDetection, prelude::*, GgrsConfig, *};
use bevy_matchbox::prelude::*;
use std::f32::consts::TAU;

use components::*;
use input::*;

mod components;
mod input;

mod arch;
use arch::ArchAppExt;

type Config = GgrsConfig<u8, PeerId>;

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum GameState {
    #[default]
    Matchmaking,
    InGame,
}

fn main() {
    let mut app = App::new();
    app.add_state::<GameState>();

    app.arch_build()
        .add_plugins(GgrsPlugin::<Config>::default())
        .rollback_component_with_clone::<Transform>()
        .rollback_component_with_copy::<Player>()
        .rollback_component_with_copy::<MoveDir>()
        .rollback_component_with_clone::<GlobalTransform>()
        .rollback_component_with_clone::<Visibility>()
        .rollback_component_with_clone::<InheritedVisibility>()
        .rollback_component_with_clone::<ViewVisibility>()
        .checksum_component::<Transform>(checksum_transform)
        .insert_resource(ClearColor(Color::rgb(0.53, 0.53, 0.53)))
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
        .add_systems(Startup, spawn_players)
        .add_systems(
            GgrsSchedule,
            (fire_bullets, move_bullet.after(fire_bullets)),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-0.5, -0.5, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

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

fn fire_bullets(
    mut commands: Commands,
    inputs: Res<PlayerInputs<Config>>,
    mut players: Query<(&Transform, &Player)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (&transform, player) in &mut players {
        let (input, _) = inputs[player.handle];
        if fire(input) {
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
