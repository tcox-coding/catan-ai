use actix_web::{rt, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_ws::AggregatedMessage;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use serde::Serializer;
// use std::borrow::{Borrow, BorrowMut};
use std::sync::{LazyLock, Mutex};
// use serde_json::{ from_str };

mod game;
use crate::game::game::Game;
use crate::game::action::Action;

static GAME: LazyLock<MutexWrapper<Game>> = LazyLock::new(|| MutexWrapper(Mutex::new(Game::new())));

#[derive(Serialize, Deserialize)]
struct WebSocketCommand {
    command: String,
    action: Option<Action>,
}

#[derive(Serialize, Deserialize)]
struct GameWrapper<'a> {
    last_action_successful: bool,
    game: Game<'a>,
}

pub struct MutexWrapper<T: ?Sized>(pub Mutex<T>);

impl<T: ?Sized + Serialize> Serialize for MutexWrapper<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0
            .lock()
            .expect("mutex is poisoned")
            .serialize(serializer)
    }
}


#[actix_web::main]
async fn main() -> std::io::Result<()>{
    HttpServer::new(|| {
        App::new()
            .route("/handleMove", web::get().to(move_handler))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

async fn move_handler(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(2_usize.pow(20));

    // start task but don't wait for it
    rt::spawn(async move {
        // receive messages from websocket
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    // echo text message
                    // session.text(text).await.unwrap();
                    let action: Result<WebSocketCommand, serde_json::Error> = serde_json::from_str(&text);
                    match action {
                        Ok(ws_command) => {
                            if ws_command.command == "new_game" {
                                GAME.0.lock().unwrap().reset();
                                let game: Game = GAME.0.lock().unwrap().clone();
                                let game_wrapper = GameWrapper{game, last_action_successful: false};
                                let game_json = serde_json::to_string(&game_wrapper).expect("Serialization failed.");
                                // Sends the game and handles the error.
                                if session.text(game_json.clone()).await.is_err() {
                                    // There was an error.
                                    println!("Error sending message.");
                                }
                            } else if ws_command.command == "take_action" {
                                let game: Game = GAME.0.lock().unwrap().clone();
                                let current_player = GAME.0.lock().unwrap().current_player_id;
                                let successful = GAME.0.lock().unwrap().takeAction(ws_command.action.unwrap(), current_player);
                                let game_wrapper = GameWrapper{game, last_action_successful: successful};
                                let game_json = serde_json::to_string(&game_wrapper).expect("Serialization failed.");
                                // Sends the game and handles the error.
                                if session.text(game_json.clone()).await.is_err() {
                                    // There was an error.
                                    println!("Error sending message.");
                                }
                            }
                        },
                        Err(error) => {
                            println!("{}", error);
                        }
                    }
                }

                Ok(AggregatedMessage::Ping(msg)) => {
                    // respond to PING frame with PONG frame
                    session.pong(&msg).await.unwrap();
                }

                _ => {}
            }
        }
    });

    // respond immediately with response connected to WS session
    Ok(res)
}