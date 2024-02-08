use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_async_task::*;
use comms::CommsSystem;
use futures::prelude::*;
use wasm_bindgen::UnwrapThrowExt;
use ws_stream_wasm::*;

mod comms;

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
        wsio.send(WsMessage::Text("HIIIIIII".into())).await.unwrap();
        info!("sent hi");
        let msg = wsio.next().await;
        info!("got result");
        let result = msg.expect_throw("Stream closed");
        info!("{:?}", result);
        return 5;
    })
}
