use bevy::prelude::*;
use bevy_ggrs::{ggrs::DesyncDetection, prelude::*, GgrsConfig, *};
use bevy_matchbox::prelude::*;
use bevy_rapier3d::prelude::*;
use std::f32::consts::TAU;

use components::*;
use input::*;

mod components;
mod input;

const PLAYER_SIZE: f32 = 1.;
const PROJECTILE_RADIUS: f32 = 0.05;

type Config = GgrsConfig<u8, PeerId>;

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum GameState {
    #[default]
    Matchmaking,
    InGame,
}

fn main() {
    let mut app = App::new();
    #[cfg(target_arch = "wasm32")]
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..default()
    }));
    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(dev::native::SizedWindowPlugin);
    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Update, dev::niceties::exits);

    app.add_plugins(GgrsPlugin::<Config>::default())
        .insert_resource(AmbientLight {
            brightness: 1.0,
            ..default()
        })
        // --
        .rollback_component_with_clone::<Transform>()
        .rollback_component_with_copy::<Player>()
        .rollback_component_with_copy::<MoveDir>()
        .rollback_component_with_clone::<GlobalTransform>()
        .rollback_component_with_clone::<Visibility>()
        .rollback_component_with_clone::<InheritedVisibility>()
        .rollback_component_with_clone::<ViewVisibility>()
        .checksum_component::<Transform>(checksum_transform)
        .insert_resource(ClearColor(Color::MIDNIGHT_BLUE * 0.1))
        .add_systems(OnEnter(GameState::Matchmaking), start_matchbox_socket)
        .add_systems(OnEnter(GameState::InGame), setup_local_players)
        .add_systems(
            Update,
            (
                wait_for_players.run_if(in_state(GameState::Matchmaking)),
                handle_ggrs_events.run_if(in_state(GameState::InGame)),
            ),
        )
        .add_systems(ReadInputs, read_local_inputs)
        .add_systems(Startup, spawn_players)
        // --
        .add_state::<GameState>()
        .add_systems(
            GgrsSchedule,
            (
                rotate_players,
                fire_projectile.after(rotate_players),
                move_projectile.after(fire_projectile),
                handle_projectile_collision.after(move_projectile),
                kill_aged_entities.after(handle_projectile_collision),
            ),
        )
        .add_systems(Startup, spawn_reference_markers)
        .insert_resource(RapierConfiguration {
            gravity: Vec3::ZERO,
            ..Default::default()
        })
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .run();
}

fn setup_local_players(
    mut commands: Commands,
    cameras: Query<Entity, With<Camera>>,
    local_players: Res<LocalPlayers>,
    players: Query<(Entity, &Player)>,
) {
    cameras.for_each(|camera| commands.entity(camera).despawn_recursive());
    for (player, &Player { handle }) in &players {
        let transform = Transform::from_translation(Vec3::new(0., 3.4, 4.0))
            .with_rotation(Quat::from_rotation_x(TAU * -0.049));
        for &local_player in &local_players.0 {
            if local_player == handle {
                commands.entity(player).with_children(|child| {
                    child.spawn(Camera3dBundle {
                        transform,
                        ..default()
                    });
                });
            }
        }
    }
}

fn handle_projectile_collision(
    mut events: EventReader<CollisionEvent>,
    projectiles: Query<&Transform, With<Projectile>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in events.read() {
        match event {
            &CollisionEvent::Started(a, b, _) => {
                let (entity, transform) = match (projectiles.get(a), projectiles.get(b)) {
                    (Err(_), Ok(transform)) => (b, *transform),
                    (Ok(transform), Err(_)) => (a, *transform),
                    (Err(_), Err(_)) => {
                        warn!("Collision of two non-projectiles");
                        return;
                    }
                    (Ok(_), Ok(_)) => {
                        warn!("Collision of two projectiles");
                        return;
                    }
                };
                commands.entity(entity).despawn_recursive();
                // "explosion"
                commands.spawn((
                    EntityTTL::new(0.2),
                    PbrBundle {
                        mesh: meshes.add(
                            Mesh::try_from(shape::Icosphere {
                                radius: PLAYER_SIZE / 5.,
                                ..Default::default()
                            })
                            .unwrap(),
                        ),
                        material: materials.add(Color::PINK.into()),
                        transform,
                        ..Default::default()
                    },
                ));
            }
            _ => {}
        }
    }
}

fn spawn_reference_markers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // origin
    commands.spawn(PbrBundle {
        mesh: meshes.add(
            Mesh::try_from(shape::Icosphere {
                radius: 0.03,
                ..Default::default()
            })
            .unwrap(),
        ),
        material: materials.add(Color::WHITE.into()),
        ..Default::default()
    });
    for value in 1..=15 {
        let value = value as f32;
        for sign in [-1., 1.] {
            for direction in [Vec3::X, Vec3::Y, Vec3::Z] {
                let transform = Transform::from_translation(direction * sign * value);
                // red, green blue for positive X, Y, Z; the complementary colors for negative
                let l = if sign > 0. { 0. } else { 1. };
                let r = l - direction.x;
                let g = l - direction.y;
                let b = l - direction.z;
                let color = Color::rgb(r, g, b);
                commands.spawn(PbrBundle {
                    mesh: meshes.add(
                        Mesh::try_from(shape::Icosphere {
                            radius: 0.025,
                            ..Default::default()
                        })
                        .unwrap(),
                    ),
                    material: materials.add(color.into()),
                    transform,
                    ..Default::default()
                });
            }
        }
    }
}

fn spawn_players(
    mut commands: Commands,
    players: Query<Entity, With<Player>>,
    projectiles: Query<Entity, With<Projectile>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for player in &players {
        commands.entity(player).despawn_recursive();
    }

    for projectile in &projectiles {
        commands.entity(projectile).despawn_recursive();
    }

    struct Attrs {
        handle: usize,
        position: Vec3,
        color: Color,
    }

    let player_attrs = [
        Attrs {
            handle: 0,
            position: Vec3::new(-2., 0., 0.),
            color: Color::GOLD,
        },
        Attrs {
            handle: 1,
            position: Vec3::new(2., 0., 0.),
            color: Color::SILVER,
        },
    ];

    struct Marker {
        position: Vec3,
        color: Color,
    }

    let marker_offset = PLAYER_SIZE / 1.8;
    let markers = &[
        Marker {
            position: Vec3::X * marker_offset,
            color: Color::RED,
        },
        Marker {
            position: Vec3::Y * marker_offset,
            color: Color::GREEN,
        },
        Marker {
            position: Vec3::Z * marker_offset,
            color: Color::BLUE,
        },
    ];

    for attr in player_attrs {
        let collider_size = PLAYER_SIZE / 2.;
        commands
            .spawn((
                RigidBody::Dynamic,
                ActiveEvents::COLLISION_EVENTS,
                Sensor::default(),
                Collider::cuboid(collider_size, collider_size, collider_size),
                Player {
                    handle: attr.handle,
                },
                PbrBundle {
                    mesh: meshes.add(
                        Mesh::try_from(shape::Cube {
                            size: PLAYER_SIZE,
                            ..Default::default()
                        })
                        .unwrap(),
                    ),
                    material: materials.add(attr.color.into()),
                    transform: Transform::from_translation(attr.position)
                        .looking_at(Vec3::ZERO, Vec3::Y),
                    ..Default::default()
                },
            ))
            // FIXME: pretty sure this was in play: https://github.com/gschup/bevy_ggrs/issues/63
            // so `.add_rollback()` to children manually.
            .with_children(|child| {
                // position markers
                for marker in markers {
                    child
                        .spawn(PbrBundle {
                            mesh: meshes.add(
                                Mesh::try_from(shape::Icosphere {
                                    radius: PLAYER_SIZE / 6.8,
                                    ..Default::default()
                                })
                                .unwrap(),
                            ),
                            material: materials.add(marker.color.into()),
                            transform: Transform::from_translation(marker.position),
                            ..Default::default()
                        })
                        .add_rollback();
                }
                // barrel
                let barrel_length = PLAYER_SIZE;
                let barrel_radius = 0.05 * PLAYER_SIZE;
                child
                    .spawn(PbrBundle {
                        mesh: meshes.add(
                            Mesh::try_from(shape::Capsule {
                                radius: barrel_radius,
                                depth: barrel_length,
                                ..Default::default()
                            })
                            .unwrap(),
                        ),
                        material: materials.add(Color::WHITE.into()),
                        transform: Transform::from_rotation(Quat::from_rotation_x(TAU / 4.0))
                            .with_translation(-Vec3::Z * barrel_length * 1.01),
                        ..Default::default()
                    })
                    .add_rollback();
            })
            .add_rollback();
    }
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
    match socket.get_channel(0) {
        Ok(_) => {}
        Err(err) => {
            error!("When trying to get channel: {err:?}");
            return;
        }
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

#[derive(Component)]
struct EntityTTL(Timer);

impl EntityTTL {
    fn new(interval: f32) -> Self {
        Self(Timer::from_seconds(interval, TimerMode::Repeating))
    }
}

fn kill_aged_entities(
    mut entities: Query<(Entity, &mut EntityTTL)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let delta = time.delta();
    for (entity, mut entity_ttl) in entities.iter_mut() {
        entity_ttl.0.tick(delta);
        if entity_ttl.0.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn handle_ggrs_events(mut session: ResMut<Session<Config>>) {
    match session.as_mut() {
        Session::P2P(s) => {
            for event in s.events() {
                match event {
                    GgrsEvent::Disconnected { .. } | GgrsEvent::NetworkInterrupted { .. } => {
                        error!("Disconnected (quitting): {event:?}");
                    }
                    _ => {
                        debug!("GgrsEvent::{event:?}");
                    }
                }
            }
        }
        _ => {}
    }
}

fn rotate_players(
    mut players: Query<(&mut Transform, &Player)>,
    inputs: Res<PlayerInputs<Config>>,
) {
    for (mut transform, player) in &mut players {
        let (input, _) = inputs[player.handle];
        let rotation = rotation(input);
        transform.rotate(Quat::from_euler(
            EulerRot::XYZ,
            rotation.x,
            rotation.y,
            rotation.z,
        ));
    }
}

fn fire_projectile(
    mut commands: Commands,
    inputs: Res<PlayerInputs<Config>>,
    mut players: Query<(&Transform, &Player)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (&transform, player) in &mut players {
        let (input, _) = inputs[player.handle];
        if fire(input) {
            let forward = -transform.local_z();
            let spawn_location = transform.translation + forward * PLAYER_SIZE * 1.65;
            commands
                .spawn((
                    EntityTTL::new(2.),
                    RigidBody::Dynamic,
                    Collider::ball(PROJECTILE_RADIUS),
                    ActiveEvents::COLLISION_EVENTS,
                    Sensor,
                    Projectile,
                    MoveDir(forward),
                    PbrBundle {
                        mesh: meshes.add(
                            Mesh::try_from(shape::Icosphere {
                                radius: PROJECTILE_RADIUS,
                                ..Default::default()
                            })
                            .unwrap(),
                        ),
                        material: materials.add(Color::RED.into()),
                        transform: Transform::from_translation(spawn_location),
                        ..Default::default()
                    },
                ))
                .add_rollback();
        }
    }
}

fn move_projectile(
    mut projectile: Query<(&mut Transform, &MoveDir), With<Projectile>>,
    time: Res<Time>,
) {
    for (mut transform, dir) in &mut projectile {
        let speed = 5.;
        let delta = dir.0 * speed * time.delta_seconds();
        transform.translation += delta;
    }
}
