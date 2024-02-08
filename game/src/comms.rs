use bevy::prelude::*;

#[derive(Event)]
enum ServerMessage {
    HELLO,
    LOBBY,
}

#[derive(Resource)]
struct ServerConnection {}

pub struct CommsSystem;
impl Plugin for CommsSystem {
    fn build(&self, app: &mut App) {}
}

pub async fn handshake() {}
