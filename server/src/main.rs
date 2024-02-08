use async_tungstenite::tokio::TokioAdapter;
use async_tungstenite::{accept_async, tungstenite::Message};
use futures::{SinkExt, StreamExt};
use std::{io, net::SocketAddr, str::FromStr};
use tokio::net::{TcpListener, TcpStream};

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
        if let Err(e) = stream.send(msg).await {
            println!("error {:}", e);
            return;
        }
    }
}
