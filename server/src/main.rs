use async_tungstenite::{accept_async, tokio::TokioAdapter, WebSocketStream};
use futures::{Future, StreamExt};
use serde::Deserialize;
use std::{
    net::SocketAddr,
    pin::Pin,
    str::FromStr,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};
use strum_macros::EnumDiscriminants;
use tokio::net::{TcpListener, TcpStream};

#[derive(Clone, Deserialize, EnumDiscriminants)]
#[strum_discriminants(name(MessageType))]
enum Message {
    Hola,
    Chao,
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from_str("127.0.0.1:3212").unwrap();
    let socket = TcpListener::bind(&addr).await.unwrap();
    loop {
        let Ok(conn) = socket.accept().await else {
            continue;
        };
        let Some(mut cl) = Client::new(conn.0, conn.1).await else {
            continue;
        };
        tokio::spawn(async move {
            cl.receive().await;
        });
    }
}

type PendingMessages = Vec<(MessageType, Box<dyn Fn(Message) -> () + Send + Sync>)>;
struct Client {
    stream: WebSocketStream<TokioAdapter<TcpStream>>,
    _addr: SocketAddr,
    pending: Arc<Mutex<PendingMessages>>,
}

impl Client {
    async fn new(stream: TcpStream, addr: SocketAddr) -> Option<Self> {
        let Ok(stream) = accept_async(TokioAdapter::new(stream)).await else {
            return None;
        };
        return Some(Client {
            stream,
            _addr: addr,
            pending: Arc::new(Mutex::new(PendingMessages::new())),
        });
    }
    async fn receive(self: &mut Self) {
        loop {
            let Some(Ok(msg)) = self.stream.next().await else {
                continue;
            };
            use async_tungstenite::tungstenite::Message;
            match msg {
                Message::Close(_) => return,
                Message::Text(text) => {
                    let pending = self.pending.clone();
                    tokio::spawn(async move { Client::on_message(text, pending).await });
                }
                _ => continue,
            }
        }
    }
    async fn on_message(raw: String, list: Arc<Mutex<PendingMessages>>) {
        let Ok(json) = serde_json::from_str::<Message>(&raw) else {
            return;
        };
        {
            let mut list = list.lock().unwrap();
            for (t, fun) in list.iter() {
                if *t == MessageType::from(&json) {
                    fun(json.clone());
                }
            }
            list.retain(|(t, _fun)| *t != MessageType::from(&json));
        }
        match json {
            Message::Hola { .. } => recibido_hola(json, list).await,
            Message::Chao { .. } => println!("xao"),
        }
    }
}

struct Pending {
    list: Arc<Mutex<PendingMessages>>,
    t: MessageType,
    out: Arc<Mutex<Option<Message>>>,
}

impl Pending {
    fn new(t: MessageType, list: Arc<Mutex<PendingMessages>>) -> Self {
        Pending {
            list,
            t,
            out: Arc::new(Mutex::new(None)),
        }
    }
}

impl Future for Pending {
    type Output = Message;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.out.lock().unwrap().as_ref() {
            None => {
                let out2 = self.out.clone();
                let cx2 = cx.waker().clone();
                self.list.lock().unwrap().push((
                    self.t,
                    Box::new(move |msg: Message| {
                        out2.lock().unwrap().replace(msg);
                        cx2.wake_by_ref();
                    }),
                ));
                Poll::Pending
            }
            Some(msg) => Poll::Ready(msg.clone()),
        }
    }
}

async fn recibido_hola(msg: Message, list: Arc<Mutex<PendingMessages>>) {
    println!("esperando el chao");
    Pending::new(MessageType::Chao, list).await;
    println!("recibido chao");
}
