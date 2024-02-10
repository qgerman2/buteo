use async_tungstenite::tokio::TokioAdapter;
use async_tungstenite::{accept_async, tungstenite::Message};
use futures::{Future, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::{io, net::SocketAddr, str::FromStr};
use strum_macros::EnumDiscriminants;
use tokio::net::{TcpListener, TcpStream};

#[derive(Clone, Debug, Serialize, Deserialize, EnumDiscriminants)]
#[strum_discriminants(name(MessageType))]
enum CustomMessage {
    Hola { x: String, y: String },
    Chao(i32, i32),
}

type ListOfWakers = Vec<(
    MessageType,
    Box<dyn Fn(CustomMessage) + Sync + Send + 'static>,
)>;

struct MyFuture {
    t: MessageType,
    list: Arc<Mutex<ListOfWakers>>,
    val: Arc<Mutex<Option<CustomMessage>>>,
}

impl Future for MyFuture {
    type Output = CustomMessage;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.val.lock().unwrap().as_ref() {
            Some(msg) => Poll::Ready(msg.clone()),
            None => {
                println!("ESTOY EN EL FUTURO");
                let cx2 = cx.waker().clone();
                let val2 = self.val.clone();
                self.list.lock().unwrap().push((
                    self.t,
                    Box::new(move |msg: CustomMessage| {
                        println!("se ha llamado el closure holy shi");
                        val2.lock().unwrap().replace(msg);
                        cx2.wake_by_ref();
                    }),
                ));
                Poll::Pending
            }
        }
    }
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

    let msgs = Arc::new(Mutex::new(ListOfWakers::new()));
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
                on_message(msgs.clone(), jj);
            } else {
                println!("error decodificando el json");
            }
        }
    }
}

fn on_message(msgs: Arc<Mutex<ListOfWakers>>, msg: CustomMessage) {
    for i in msgs.lock().unwrap().iter() {
        if i.0 == MessageType::from(&msg) {
            println!("encontrado que se esta esperando un chao!!!!");
            i.1.as_ref()(msg.clone());
        }
    }
    msgs.lock()
        .unwrap()
        .retain(|i| i.0 != MessageType::from(&msg));
    match msg {
        CustomMessage::Hola { .. } => on_hola(msgs),
        CustomMessage::Chao(..) => println!("chao"),
    }
}

fn on_hola(msgs: Arc<Mutex<ListOfWakers>>) {
    println!("hola");
    tokio::spawn(wait_for_chao(msgs));
}

async fn wait_for(t: MessageType, msgs: Arc<Mutex<ListOfWakers>>) -> CustomMessage {
    let aver = MyFuture {
        t,
        list: msgs,
        val: Arc::new(Mutex::new(None)),
    };
    aver.await
}

async fn wait_for_chao(msgs: Arc<Mutex<ListOfWakers>>) {
    println!("waiting for chao");
    let fuutuuro = wait_for(MessageType::Chao, msgs).await;
    println!("TERMINO EL AWAIT")
}
