use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_async_task::*;
use comms::CommsSystem;
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::UnwrapThrowExt;
use ws_stream_wasm::*;

mod comms;

#[derive(Debug, Serialize, Deserialize)]
enum CustomMessage {
    Hola { x: String, y: String },
    Chao(i32, i32),
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    filter: "info,wgpu_core=error,wgpu_hal=error,bevy_render=error".into(),
                    level: bevy::log::Level::INFO,
                }),
        )
        .add_systems(Startup, my_system)
        .run();
}

fn my_system(mut task_executor: AsyncTaskRunner<u32>) {
    task_executor.start(async {
        let (_ws, mut wsio) = WsMeta::connect("ws://127.0.0.1:3212", None).await.unwrap();
        info!("holy shi");
        let mimensage = CustomMessage::Chao(10, 13);
        let msg = serde_json::to_string(&mimensage).unwrap();
        wsio.send(WsMessage::Text(msg)).await.unwrap();
        info!("sent msg");
        let msg = wsio.next().await;
        info!("got result");
        let result = msg.expect_throw("Stream closed");
        if let WsMessage::Text(tex) = result {
            if let Ok(respuesta) = serde_json::from_str::<CustomMessage>(&tex) {
                info!("got json {:?}", respuesta);
            } else {
                info!("error al leer json");
            }
        } else {
            info!("message received is not text");
        }
        return 5;
    })
}
