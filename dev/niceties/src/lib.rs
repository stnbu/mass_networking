use bevy::app::AppExit;
use bevy::prelude::*;

pub fn exits(
    /* mut session: ResMut<Session<T>>, */
    keys: Res<Input<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    /* no for now **
    match session.as_mut() {
        Session::P2P(s) => {
            for event in s.events() {
                match event {
                    GgrsEvent::Disconnected { .. } | GgrsEvent::NetworkInterrupted { .. } => {
                        error!("Disconnected (quitting)");
                        exit.send(AppExit);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    */
    if keys.pressed(KeyCode::Escape) {
        info!("Exit on keypress");
        exit.send(AppExit);
    }
}
