use crate::model::{ClientCommand, GameView};
use dioxus::{fullstack::ServerEvents, prelude::*};

#[cfg(feature = "server")]
use crate::{
    ai::{ask_ai_for_turn, fallback_ai_plan},
    game::Game,
};
#[cfg(feature = "server")]
use std::{
    fmt::Display,
    sync::{Arc, OnceLock},
};
#[cfg(feature = "server")]
use tokio::sync::{Mutex, broadcast};

#[cfg(feature = "server")]
pub struct AppState {
    game: Arc<Mutex<Game>>,
    changed: broadcast::Sender<()>,
}

#[cfg(feature = "server")]
static STATE: OnceLock<AppState> = OnceLock::new();

#[cfg(feature = "server")]
pub fn init_state() {
    let _ = STATE.set(AppState {
        game: Arc::new(Mutex::new(Game::new())),
        changed: broadcast::channel(64).0,
    });
}

#[cfg(feature = "server")]
fn state() -> Result<&'static AppState, ServerFnError> {
    STATE
        .get()
        .ok_or_else(|| ServerFnError::new("game server state was not initialized"))
}

#[cfg(feature = "server")]
fn server_error(error: impl Display) -> ServerFnError {
    ServerFnError::new(error)
}

#[get("/api/game/events")]
pub async fn stream_game_events(
    viewer_id: Option<u64>,
) -> Result<ServerEvents<GameView>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let state = state()?;
        let mut rx = state.changed.subscribe();
        let initial = {
            let game = state.game.lock().await;
            game.view_for(viewer_id)
        };
        let game = state.game.clone();

        Ok(ServerEvents::new(move |mut tx| async move {
            if tx.send(initial).await.is_err() {
                return;
            }
            loop {
                match rx.recv().await {
                    Ok(()) => {
                        let snapshot = {
                            let game = game.lock().await;
                            game.view_for(viewer_id)
                        };
                        if tx.send(snapshot).await.is_err() {
                            return;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => return,
                }
            }
        }))
    }
    #[cfg(not(feature = "server"))]
    unreachable!()
}

#[post("/api/game/join")]
pub async fn join_human(name: String) -> Result<(u64, GameView), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let state = state()?;
        let mut game = state.game.lock().await;
        let id = game.add_player(clean_name(&name), false);
        let view = game.view_for(Some(id));
        let _ = state.changed.send(());
        Ok((id, view))
    }
    #[cfg(not(feature = "server"))]
    unreachable!()
}

#[post("/api/game/command")]
pub async fn send_game_command(
    viewer_id: Option<u64>,
    command: ClientCommand,
) -> Result<GameView, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let state = state()?;
        match command {
            ClientCommand::JoinHuman { name } => {
                let mut game = state.game.lock().await;
                let id = game.add_player(clean_name(&name), false);
                let view = game.view_for(Some(id));
                let _ = state.changed.send(());
                Ok(view)
            }
            ClientCommand::AddAi => {
                let mut game = state.game.lock().await;
                let number = game.player_count() + 1;
                let id = game.add_player(format!("AI Wizard {number}"), true);
                game.private_log(
                    id,
                    "You are an AI player. Private hand and reagents are visible only to you."
                        .into(),
                );
                let view = game.view_for(viewer_id);
                let _ = state.changed.send(());
                Ok(view)
            }
            ClientCommand::SendChat { player_id, message } => {
                let mut game = state.game.lock().await;
                game.send_chat(player_id, message).map_err(server_error)?;
                let view = game.view_for(viewer_id);
                let _ = state.changed.send(());
                Ok(view)
            }
            ClientCommand::MoveStep { player_id, target } => {
                let mut game = state.game.lock().await;
                game.move_step(player_id, target).map_err(server_error)?;
                let view = game.view_for(viewer_id);
                let _ = state.changed.send(());
                Ok(view)
            }
            ClientCommand::SubmitPlan { player_id, plan } => {
                submit_plan_impl(state, viewer_id, player_id, plan).await?;
                auto_ai_turns(state).await?;
                snapshot(state, viewer_id).await
            }
            ClientCommand::AskAiTurn { player_id } => {
                let view = ai_view(state, player_id).await?;
                let plan = ask_ai_for_turn(&view)
                    .await
                    .unwrap_or_else(|error| fallback_ai_plan(&view, player_id, error));
                submit_plan_impl(state, viewer_id, player_id, plan).await
            }
        }
    }
    #[cfg(not(feature = "server"))]
    unreachable!()
}

#[cfg(feature = "server")]
async fn submit_plan_impl(
    state: &AppState,
    viewer_id: Option<u64>,
    player_id: u64,
    plan: crate::model::TurnPlan,
) -> Result<GameView, ServerFnError> {
    let mut game = state.game.lock().await;
    game.submit_plan(player_id, plan).map_err(server_error)?;
    let view = game.view_for(viewer_id);
    let _ = state.changed.send(());
    Ok(view)
}

#[cfg(feature = "server")]
async fn auto_ai_turns(state: &AppState) -> Result<(), ServerFnError> {
    let start_day = {
        let game = state.game.lock().await;
        game.day()
    };
    loop {
        let next_ai = {
            let game = state.game.lock().await;
            if game.day() != start_day {
                return Ok(());
            }
            game.next_unready_ai()
        };
        let Some(player_id) = next_ai else {
            return Ok(());
        };
        let view = {
            let game = state.game.lock().await;
            game.view_for(Some(player_id))
        };
        let plan = ask_ai_for_turn(&view)
            .await
            .unwrap_or_else(|error| fallback_ai_plan(&view, player_id, error));
        let mut game = state.game.lock().await;
        game.submit_plan(player_id, plan).map_err(server_error)?;
        let _ = state.changed.send(());
    }
}

#[cfg(feature = "server")]
async fn snapshot(state: &AppState, viewer_id: Option<u64>) -> Result<GameView, ServerFnError> {
    let game = state.game.lock().await;
    Ok(game.view_for(viewer_id))
}

#[cfg(feature = "server")]
async fn ai_view(state: &AppState, player_id: u64) -> Result<GameView, ServerFnError> {
    let game = state.game.lock().await;
    if !game.is_ai_player(player_id) {
        return Err(server_error("That player is not an AI."));
    }
    Ok(game.view_for(Some(player_id)))
}

#[cfg(feature = "server")]
fn clean_name(name: &str) -> String {
    let clean = name.trim();
    if clean.is_empty() {
        "Unnamed Wizard".into()
    } else {
        clean.chars().take(32).collect()
    }
}
