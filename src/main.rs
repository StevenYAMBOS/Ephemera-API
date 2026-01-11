use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use rand::{distr::Alphanumeric, prelude::*};
use std::sync::Arc;
use tokio::sync::broadcast;

// Génération de l'id
fn get_rand_string() -> String {
    let s: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(3)
        .map(char::from)
        .collect();
    println!("{}", s);
    s
}

// `broadcast` ce channel (`tokio::sync::broadcast`) permet la communication multi-client en temps réel
// On le met dans ce `state` pour le rendre accessible par n'importe quel handler
struct AppState {
    tx: broadcast::Sender<String>,
}

// Page d'accueil
async fn home() -> String {
    // Idée :
    // Quand on GET le endpoint on "POST" l'id
    // Requete + reponse :
    // Requete du enpoint qui demande l'id -> response qui donne l'id
    get_rand_string()
}

// Handler websocket
async fn handle_ws_upgrade(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

// Logique socket
async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(t)) => {
                println!("Texte reçu : {}", t);
                // Echo message back or broadcast
                socket
                    .send(Message::Text(format!("Echo: {}", t).into()))
                    .await
                    .unwrap();
            }
            Ok(Message::Binary(b)) => {
                println!("Binaire reçu : {:?}", b);
            }
            Ok(Message::Ping(_)) => {
                println!("Ping reçu ! ");
            }
            Ok(Message::Close(_)) => {
                println!("Le client est déconnecté");
                break; // Exit loop on close
            }
            Ok(axum::extract::ws::Message::Pong(_)) => {
                todo!()
            }
            Err(e) => {
                eprintln!("Erreur websocket : {:?}", e);
                break; // Exit loop on error
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let (tx, _rx) = broadcast::channel(100); // Le Buffer peut aller jusqu'à 100 messages
    let app_state = Arc::new(AppState { tx });
    let app = Router::new()
        .route("/ws", get(handle_ws_upgrade))
        .route("/{id}", get(home))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!(
        "Le serveur est lançé sur {}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}
