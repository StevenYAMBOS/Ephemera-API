use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use std::sync::Arc;
use tokio::sync::broadcast;

// `broadcast` ce channel (`tokio::sync::broadcast`) permet la communication multi-client en temps réel
// On le met dans ce `state` pour le rendre accessible par n'importe quel handler
struct AppState {
    tx: broadcast::Sender<String>,
}

async fn handle_ws_upgrade(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(t)) => {
                println!("Received text: {}", t);
                // Echo message back or broadcast
                socket
                    .send(Message::Text(format!("Echo: {}", t).into()))
                    .await
                    .unwrap();
            }
            Ok(Message::Binary(b)) => {
                println!("Received binary: {:?}", b);
            }
            Ok(Message::Ping(_)) => {
                println!("Received ping");
            }
            Ok(Message::Close(_)) => {
                println!("Client disconnected");
                break; // Exit loop on close
            }
            Ok(axum::extract::ws::Message::Pong(_)) => {
                todo!()
            }
            Err(e) => {
                eprintln!("WebSocket error: {:?}", e);
                break; // Exit loop on error
            }
        }
    }
}

/*
async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // client disconnected
            return;
        };
        if socket.send(msg).await.is_err() {
            // client disconnected
            return;
        }
    }
}
*/

#[tokio::main]
async fn main() {
    let (tx, _rx) = broadcast::channel(100); // Le Buffer peut aller jusqu'à 100 messages
    let app_state = Arc::new(AppState { tx });
    let app = Router::new()
        .route("/ws", get(handle_ws_upgrade))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
