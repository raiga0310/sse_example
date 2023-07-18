/* use axum::{routing::get, Router, response::IntoResponse};
use futures::stream::{Stream, repeat};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;

use std::collections::HashMap;
use std::time::Duration;
use tokio::time::interval;
use hyper::Body;
use http::{Response, header};

struct Events {
    clients: Arc<Mutex<HashMap<u64, tokio::sync::mpsc::UnboundedSender<String>>>>,
    last_id: u64,
}

impl Events {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            last_id: 0,
        }
    }

    pub async fn subscribe(&mut self) -> tokio::sync::mpsc::UnboundedReceiver<String> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        self.last_id += 1;
        self.clients.lock().await.insert(self.last_id, tx);
        rx
    }

    pub async fn notify(&self, msg: String) {
        let mut clients = self.clients.lock().await;
        clients.retain(|_, sender| sender.send(msg.clone()).is_ok());
    }
}

static EVENTS: once_cell::sync::Lazy<Events> = once_cell::sync::Lazy::new(Events::new);

async fn sse() -> impl IntoResponse {
    let rx = EVENTS.subscribe().await;
    let stream = UnboundedReceiverStream::new(rx).map(|msg| Ok::<_, hyper::Error>(format!("data: {}\n\n", msg)));

    
    Response::builder()
        .header(header::CONTENT_TYPE, "text/event-stream")
        .body(Body::wrap_stream(stream))
        .unwrap()
}

async fn ping() -> &'static str {
    EVENTS.notify("pong!".to_string()).await;
    "sent ping!"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/events", get(sse))
        .route("/ping", get(ping));
    
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
*/

use axum::{routing::get, Router, response::IntoResponse};
use tower_http::cors::{Any, CorsLayer};
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use tokio::sync::Mutex;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;

use std::collections::HashMap;
use hyper::Body;
use http::{Response, header};

struct Events {
    clients: Arc<Mutex<HashMap<u64, tokio::sync::mpsc::UnboundedSender<String>>>>,
    last_id: AtomicU64,
}

impl Events {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            last_id: AtomicU64::new(0),
        }
    }

    pub async fn subscribe(&self) -> tokio::sync::mpsc::UnboundedReceiver<String> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let id = self.last_id.fetch_add(1, Ordering::SeqCst);
        self.clients.lock().await.insert(id, tx);
        rx
    }

    pub async fn notify(&self, msg: String) {
        let mut clients = self.clients.lock().await;
        clients.retain(|_, sender| sender.send(msg.clone()).is_ok());
    }
}

static EVENTS: once_cell::sync::Lazy<Arc<Events>> = once_cell::sync::Lazy::new(|| Arc::new(Events::new()));

async fn sse(events: Arc<Events>) -> impl IntoResponse {
    let rx = events.subscribe().await;
    let stream = UnboundedReceiverStream::new(rx).map(|msg| Ok::<_, hyper::Error>(format!("data: {}\n\n", msg)));

    
    Response::builder()
        .header(header::CONTENT_TYPE, "text/event-stream")
        .body(Body::wrap_stream(stream))
        .unwrap()
}

async fn ping(events: Arc<Events>) -> &'static str {
    events.notify("pong!".to_string()).await;
    "sent ping!"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/events", get({
            let events = Arc::clone(&EVENTS);
            move || sse(Arc::clone(&events))
        }))
        .route("/ping", get({
            let events = Arc::clone(&EVENTS);
            move || ping(Arc::clone(&events))
        }))
        // add cors any
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
        );

    
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
