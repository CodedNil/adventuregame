use crate::model::*;
use reqwest::Client;
use serde_json::json;
use std::env;

pub async fn ask_ai_for_turn(view: &GameView) -> Result<TurnPlan, String> {
    let api_key =
        env::var("OPENROUTER_API_KEY").map_err(|_| "OPENROUTER_API_KEY is not set".to_string())?;
    let model =
        env::var("OPENROUTER_MODEL").unwrap_or_else(|_| "deepseek/deepseek-v4-flash".into());
    let own_id = view.viewer_id.ok_or("AI view has no player id")?;
    let response = Client::new()
        .post("https://openrouter.ai/api/v1/responses")
        .bearer_auth(api_key)
        .header("Content-Type", "application/json")
        .header("X-OpenRouter-Title", "The Unreliable Fellowship Prototype")
        .json(&json!({
            "model": model,
            "instructions": AI_SYSTEM_PROMPT,
            "input": build_ai_prompt(view),
            "temperature": 0.8,
            "max_output_tokens": 700,
            "text": {
                "format": {
                    "type": "json_schema",
                    "name": "turn_plan",
                    "schema": ai_plan_schema()
                }
            }
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Err(format!("OpenRouter returned {}", response.status()));
    }

    let body: serde_json::Value = response.json().await.map_err(|error| error.to_string())?;
    let text = body
        .get("output_text")
        .and_then(|value| value.as_str())
        .or_else(|| {
            body.get("output")
                .and_then(|output| output.as_array())
                .and_then(|items| items.first())
                .and_then(|item| item.get("content"))
                .and_then(|content| content.as_array())
                .and_then(|items| items.first())
                .and_then(|item| item.get("text"))
                .and_then(|value| value.as_str())
        })
        .ok_or_else(|| format!("OpenRouter response did not contain text: {body}"))?;

    parse_ai_plan(text, view, own_id)
}

pub fn fallback_ai_plan(view: &GameView, player_id: u64, reason: String) -> TurnPlan {
    let player = view.players.iter().find(|player| player.id == player_id);
    let target = view
        .tiles
        .iter()
        .filter(|tile| tile.completed_by.is_none() && tile.sealed_days == 0)
        .min_by_key(|tile| {
            player
                .map(|player| movement_cost(&player.pos, &tile.pos, view.caravan_row))
                .unwrap_or(0)
        })
        .map(|tile| tile.pos.clone())
        .unwrap_or(Position {
            row: view.caravan_row,
            col: 0,
        });
    TurnPlan {
        target,
        event_action: EventAction::Attempt,
        card_action: CardAction::None,
        chat: reason,
    }
}

fn build_ai_prompt(view: &GameView) -> String {
    serde_json::to_string_pretty(&json!({
        "request": "Choose your next turn. You may scheme in chat, but the returned JSON must be a legal turn plan.",
        "visible_state": view,
        "legal_notes": {
            "movement": "Pick any visible row/column. Server clamps illegal coordinates and charges energy.",
            "event_action": ["Attempt", "Rest", "Ignore"],
            "card_action": ["None", "PoisonedBreakfast", "AlarmWard", "EventSeal", "SpellOfTruth"],
            "hidden_info": "You only see your own hand/reagents and public info."
        }
    }))
    .unwrap_or_default()
}

fn ai_plan_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "target_row": { "type": "integer" },
            "target_col": { "type": "integer" },
            "event_action": { "type": "string", "enum": ["Attempt", "Rest", "Ignore"] },
            "card_action": { "type": "string", "enum": ["None", "PoisonedBreakfast", "AlarmWard", "EventSeal", "SpellOfTruth"] },
            "target_player_id": { "type": "integer" },
            "question": { "type": "string" },
            "chat": { "type": "string" }
        },
        "required": ["target_row", "target_col", "event_action", "card_action", "chat"]
    })
}

fn parse_ai_plan(text: &str, view: &GameView, own_id: u64) -> Result<TurnPlan, String> {
    let value: serde_json::Value = serde_json::from_str(text).map_err(|error| error.to_string())?;
    let row = value
        .get("target_row")
        .and_then(|value| value.as_i64())
        .unwrap_or(view.caravan_row as i64) as i32;
    let col = value
        .get("target_col")
        .and_then(|value| value.as_i64())
        .unwrap_or(0) as i32;
    let event_action = match value.get("event_action").and_then(|value| value.as_str()) {
        Some("Rest") => EventAction::Rest,
        Some("Ignore") => EventAction::Ignore,
        _ => EventAction::Attempt,
    };
    let target_player_id = value
        .get("target_player_id")
        .and_then(|value| value.as_u64())
        .unwrap_or_else(|| {
            view.players
                .iter()
                .find(|player| player.id != own_id)
                .map(|player| player.id)
                .unwrap_or(own_id)
        });
    let question = value
        .get("question")
        .and_then(|value| value.as_str())
        .unwrap_or("What are you planning?")
        .to_string();
    let card_action = match value.get("card_action").and_then(|value| value.as_str()) {
        Some("PoisonedBreakfast") => CardAction::PlayCard {
            card: Card::PoisonedBreakfast,
            target: CardTarget::Player(target_player_id),
            question: None,
        },
        Some("AlarmWard") => CardAction::PlayCard {
            card: Card::AlarmWard,
            target: CardTarget::None,
            question: None,
        },
        Some("EventSeal") => CardAction::PlayCard {
            card: Card::EventSeal,
            target: CardTarget::Tile(Position { row, col }),
            question: None,
        },
        Some("SpellOfTruth") => CardAction::PlayCard {
            card: Card::SpellOfTruth,
            target: CardTarget::Player(target_player_id),
            question: Some(question),
        },
        _ => CardAction::None,
    };
    Ok(TurnPlan {
        target: Position { row, col },
        event_action,
        card_action,
        chat: value
            .get("chat")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .chars()
            .take(240)
            .collect(),
    })
}

const AI_SYSTEM_PROMPT: &str = r#"You are playing The Unreliable Fellowship, a web board/card game about bumbling wizards on a 20-day caravan quest.

Rules summary:
- You are one wizard. One player wins by having the most Glory at the end.
- The board is a moving road. The caravan moves forward 2 rows after each day.
- You can move sideways or forward. Farther movement and being ahead of the caravan costs more Energy.
- Event tiles can be completed once, so claiming valuable events denies them to others.
- Cooperation and promises are social, not enforced. Betrayal is allowed.
- Sabotage is usually anonymous. You may scheme unless a truth effect forces honesty.
- You only know public state plus your own private hand/reagents.
- Hard rules are enforced by the server; choose a sensible legal plan from what you can see.

Return only JSON matching the requested schema. Chat should be table talk visible to others. You can lie, bargain, threaten, or stay quiet unless responding to a Spell of Truth prompt."#;
