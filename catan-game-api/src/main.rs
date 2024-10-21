use actix_web::{web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use actix_ws::Message;
use futures::stream::StreamExt;
use std::sync::{LazyLock, Mutex};

mod game;
use crate::game::game::Game;

static GAME: LazyLock<Mutex<Game>> = LazyLock::new(|| Mutex::new(Game::new()));

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    HttpServer::new(|| {
        App::new()
            .route("/ping", web::get().to(pong))
            .route("/handleMove", web::get().to(move_handler))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}


async fn pong() -> impl Responder {
    HttpResponse::Ok().body("pong")
}

// Websocket handler 
async fn move_handler(req: HttpRequest, body: web::Payload) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    // start task but don't wait for it
    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }
                Message::Text(msg) => println!("Got text: {msg}"),
                _ => break,
            }
        }

        let _ = session.close(None).await;
    });

    // respond immediately with response connected to WS session
    Ok(response)
}