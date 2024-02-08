use async_tungstenite::tokio::TokioAdapter;
use async_tungstenite::{accept_async, tungstenite::Message};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{io, net::SocketAddr, str::FromStr};
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug, Serialize, Deserialize)]
enum CustomMessage {
    Hola { x: String, y: String },
    Chao(i32, i32),
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from_str("127.0.0.1:3212").unwrap();
    println!("server task listening at: {}", &addr);
    let socket = TcpListener::bind(&addr).await.unwrap();
    loop {
        tokio::spawn(connection(socket.accept().await));
    }
}

async fn connection(stream: Result<(TcpStream, SocketAddr), io::Error>) {
    let Ok((stream, addr)) = stream else { return };
    let Ok(mut stream) = accept_async(TokioAdapter::new(stream)).await else {
        return;
    };
    println!("Conectado {}", addr);
    loop {
        let Some(Ok(msg)) = stream.next().await else {
            println!("no se recibio nada");
            continue;
        };
        println!("recibido {:?}", msg);
        if let Message::Close(_) = msg {
            println!("conexion desconectada por remoto");
            return;
        }
        if let Message::Text(tex) = msg {
            if let Ok(jj) = serde_json::from_str::<CustomMessage>(&tex.to_string()) {
                println!("got json {:?}", jj);
            } else {
                println!("error decodificando el json");
            }
        }
        let mimensaje = CustomMessage::Hola {
            x: "miau".into(),
            y: "miu".into(),
        };
        let msg = Message::Text(serde_json::to_string(&mimensaje).unwrap());
        if let Err(e) = stream.send(msg).await {
            println!("error {:}", e);
            return;
        }
    }
}
