use crate::cards::{TargetKind, card_cost_label, card_def};
use crate::model::*;
use crate::server_functions::{join_human, send_game_command, stream_game_events};
use crate::styles;
use dioxus::prelude::*;

#[allow(non_snake_case)]
#[component]
pub fn App() -> Element {
    let mut view = use_signal(|| None::<GameView>);
    let mut player_id = use_signal(|| None::<u64>);
    let mut name = use_signal(|| "Mirthwick".to_string());
    let mut target_row = use_signal(|| 0_i32);
    let mut target_col = use_signal(|| BOARD_WIDTH / 2);
    let mut selected_card = use_signal(|| None::<Card>);
    let mut target_player = use_signal(|| None::<u64>);
    let mut truth_question = use_signal(|| "What are you planning?".to_string());
    let mut chat = use_signal(String::new);

    use_effect(move || {
        let viewer_id = *player_id.read();
        spawn(async move {
            let Ok(mut events) = stream_game_events(viewer_id).await else {
                return;
            };
            while let Some(Ok(snapshot)) = events.recv().await {
                if *player_id.read() == viewer_id {
                    view.set(Some(snapshot));
                }
            }
        });
    });

    let snapshot = view.read().clone();
    let own_player = snapshot.as_ref().and_then(|snapshot| snapshot.own.clone());
    let players = snapshot
        .as_ref()
        .map(|snapshot| snapshot.players.clone())
        .unwrap_or_default();
    let current_player = player_id
        .read()
        .and_then(|id| players.iter().find(|player| player.id == id).cloned());
    let selected_position = Position {
        row: *target_row.read(),
        col: *target_col.read(),
    };

    rsx! {
        style { "{styles::BASE_CSS}" }
        main { style: styles::SHELL,
            section { style: styles::TOPBAR,
                div {
                    h1 { "The Unreliable Fellowship" }
                    p { "Day {snapshot.as_ref().map(|s| s.day).unwrap_or(1)} of {QUEST_DAYS} | caravan row {snapshot.as_ref().map(|s| s.caravan_row).unwrap_or(0)}" }
                }
                div { style: styles::JOIN,
                    input {
                        value: "{name}",
                        oninput: move |event| name.set(event.value())
                    }
                    button {
                        onclick: move |_| async move {
                            if let Ok((id, snapshot)) = join_human(name.read().clone()).await {
                                if let Some(player) = snapshot.players.iter().find(|player| player.id == id) {
                                    target_row.set(player.pos.row);
                                    target_col.set(player.pos.col);
                                }
                                player_id.set(Some(id));
                                view.set(Some(snapshot));
                            }
                        },
                        "Join"
                    }
                    button {
                        onclick: move |_| async move {
                            if let Ok(snapshot) = send_game_command(*player_id.read(), ClientCommand::AddAi).await {
                                view.set(Some(snapshot));
                            }
                        },
                        "Add AI"
                    }
                }
            }

            if let Some(snapshot) = snapshot.clone() {
                section { class: "responsive-layout", style: styles::LAYOUT,
                    div { style: styles::BOARD_PANEL,
                        div { style: styles::BOARD,
                            for row in ((snapshot.caravan_row - 1)..=(snapshot.caravan_row + FORWARD_ROWS)).rev() {
                                div { style: styles::BOARD_ROW,
                                    div { style: styles::ROW_LABEL,
                                        if row > snapshot.caravan_row {
                                            "Future"
                                        } else if row == snapshot.caravan_row {
                                            "Caravan"
                                        } else {
                                            "Behind"
                                        }
                                        span { style: styles::ROW_LABEL_SUB, " r{row}" }
                                    }
                                    for col in 0..snapshot.width {
                                        InteractiveBoardTile {
                                            row,
                                            col,
                                            tile: snapshot.tiles.iter().find(|tile| tile.pos.row == row && tile.pos.col == col).cloned(),
                                            players: snapshot.players.iter().filter(|player| player.pos.row == row && player.pos.col == col).cloned().collect(),
                                            current_player: current_player.clone(),
                                            caravan_row: snapshot.caravan_row,
                                            selected: *target_row.read() == row && *target_col.read() == col,
                                            view,
                                            player_id,
                                            target_row,
                                            target_col,
                                        }
                                    }
                                }
                            }
                        }
                    }

                    aside { class: "side-panel", style: styles::SIDE,
                        TurnPanel {
                            current_player,
                            selected_position: selected_position.clone(),
                            selected_card: *selected_card.read(),
                            target_player: *target_player.read(),
                            truth_question: truth_question.read().clone(),
                            players: players.clone(),
                            view,
                            player_id,
                            set_selected_card: move |card| selected_card.set(card),
                            set_target_player: move |id| target_player.set(id),
                            set_truth_question: move |question| truth_question.set(question),
                        }

                        ChatPanel {
                            chat: chat.read().clone(),
                            set_chat: move |message| chat.set(message),
                            submit: move |_| async move {
                                if let Some(id) = *player_id.read() {
                                    let chat_message = chat.read().clone();
                                    if let Ok(snapshot) = send_game_command(*player_id.read(), ClientCommand::SendChat {
                                        player_id: id,
                                        message: chat_message,
                                    }).await {
                                        chat.set(String::new());
                                        view.set(Some(snapshot));
                                    }
                                }
                            }
                        }

                        PlayersPanel {
                            players: snapshot.players.clone(),
                        }
                    }
                }

                if let Some(own) = own_player {
                    HandPanel {
                        own,
                        selected_card: *selected_card.read(),
                        set_selected_card: move |card| selected_card.set(card),
                    }
                }

                section { style: styles::LOG_PANEL,
                    h2 { "Log" }
                    for entry in snapshot.logs {
                        p { style: styles::LOG_ENTRY, class: if entry.private_to.is_some() { "private-log" } else { "" },
                            "Day {entry.day}: {entry.text}"
                        }
                    }
                }

                if !snapshot.winners.is_empty() {
                    div { style: styles::MODAL,
                        div { style: styles::MODAL_BOX,
                            h2 { "Quest Complete" }
                            p { "Winner: {snapshot.winners.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join(\", \")}" }
                        }
                    }
                }
            } else {
                section { style: styles::EMPTY, "Connecting to the game server..." }
            }
        }
    }
}

#[component]
fn TurnPanel(
    current_player: Option<PublicPlayer>,
    selected_position: Position,
    selected_card: Option<Card>,
    target_player: Option<u64>,
    truth_question: String,
    players: Vec<PublicPlayer>,
    view: Signal<Option<GameView>>,
    player_id: Signal<Option<u64>>,
    set_selected_card: EventHandler<Option<Card>>,
    set_target_player: EventHandler<Option<u64>>,
    set_truth_question: EventHandler<String>,
) -> Element {
    let selected_def = selected_card.map(card_def);
    let needs_player = selected_def.is_some_and(|def| def.target == TargetKind::Player);
    let needs_tile = selected_def.is_some_and(|def| def.target == TargetKind::Tile);
    let missing_required_target = needs_player && target_player.is_none();

    rsx! {
        div { style: styles::PANEL,
            h2 { "Turn" }
            if let Some(player) = current_player {
                p { style: styles::MUTED, "Playing as {player.name}. Energy {player.energy}, Glory {player.glory}." }
                div { style: styles::MOVE_SUMMARY,
                    strong { "Position" }
                    span { "Row {player.pos.row}, column {player.pos.col}" }
                    span { "Click an adjacent map tile to move 1 step immediately." }
                }
                if let Some(card) = selected_card {
                    div { style: styles::SELECTED_CARD,
                        strong { "{card_name(&card)}" }
                        span { "{card_def(card).rules}" }
                        span { "Cost: {card_cost_label(card)}" }
                    }
                    if needs_player {
                        label { style: styles::LABEL, "Target" }
                        div { style: styles::TARGET_LIST,
                            for target in players.clone().into_iter().filter(|target| target.id != player.id) {
                                button {
                                    class: if target_player == Some(target.id) { "selected" } else { "" },
                                    onclick: move |_| set_target_player.call(Some(target.id)),
                                    "{target.name}"
                                }
                            }
                        }
                    }
                    if matches!(card, Card::SpellOfTruth) {
                        label { style: styles::LABEL, "Question" }
                        input {
                            style: styles::FULL_CONTROL,
                            value: "{truth_question}",
                            oninput: move |event| set_truth_question.call(event.value())
                        }
                    }
                } else {
                    p { style: styles::MUTED, "Select a card from your hand to play one this turn." }
                }
                button {
                    class: "primary",
                    disabled: missing_required_target,
                    onclick: move |_| {
                        let selected_position = selected_position.clone();
                        let truth_question = truth_question.clone();
                        let current_pos = player.pos.clone();
                        async move {
                        if let Some(id) = *player_id.read() {
                            let action = selected_card
                                .map(|card| CardAction::PlayCard {
                                    card,
                                    target: card_target(card, target_player, selected_position.clone()),
                                    question: if matches!(card, Card::SpellOfTruth) { Some(truth_question.clone()) } else { None },
                                })
                                .unwrap_or(CardAction::None);
                            if let Ok(snapshot) = send_game_command(*player_id.read(), ClientCommand::SubmitPlan {
                                    player_id: id,
                                    plan: TurnPlan {
                                    target: current_pos.clone(),
                                    event_action: EventAction::Attempt,
                                    card_action: action,
                                    chat: String::new(),
                                },
                            }).await {
                                set_selected_card.call(None);
                                view.set(Some(snapshot));
                            }
                        }
                        }
                    },
                    if missing_required_target {
                        "Pick Target First"
                    } else if needs_tile {
                        "Play Card And End Turn"
                    } else {
                        "End Turn"
                    }
                }
            } else {
                p { style: styles::MUTED, "Join as a human player or add AI players." }
            }
        }
    }
}

#[component]
fn ChatPanel(
    chat: String,
    set_chat: EventHandler<String>,
    submit: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div { style: styles::PANEL,
            h2 { "Chat" }
            textarea {
                style: styles::FULL_CONTROL,
                value: "{chat}",
                placeholder: "Send a public message...",
                oninput: move |event| set_chat.call(event.value())
            }
            button {
                onclick: move |event| submit.call(event),
                "Send"
            }
        }
    }
}

#[component]
fn PlayersPanel(players: Vec<PublicPlayer>) -> Element {
    rsx! {
        div { style: styles::PANEL,
            h2 { "Players" }
            for player in players {
                div { style: styles::PLAYER_LINE,
                    strong { "{player.name}" }
                    span { style: styles::SMALL_TEXT, if player.is_ai { "AI" } else { "Human" } }
                    span { style: styles::SMALL_TEXT, "E {player.energy}" }
                    span { style: styles::SMALL_TEXT, "G {player.glory}" }
                    span { style: styles::SMALL_TEXT, if player.ready { "Ready" } else { "Planning" } }
                }
            }
        }
    }
}

#[component]
fn HandPanel(
    own: OwnPlayer,
    selected_card: Option<Card>,
    set_selected_card: EventHandler<Option<Card>>,
) -> Element {
    rsx! {
        div { style: styles::HAND_BAR,
            h2 { "Hand" }
            div { style: styles::SMALL_TEXT, "Ash {own.reagents.ash} | Mooncap {own.reagents.mooncap} | Ichor {own.reagents.ichor} | Quartz {own.reagents.quartz}" }
            div { style: styles::HAND_GRID,
                for card in own.hand {
                    GameCard {
                        card,
                        selected: selected_card == Some(card),
                        onclick: move |_| set_selected_card.call(Some(card)),
                    }
                }
            }
        }
    }
}

#[component]
fn GameCard(card: Card, selected: bool, onclick: EventHandler<MouseEvent>) -> Element {
    let def = card_def(card);
    rsx! {
        button {
            class: if selected { "selected" } else { "" },
            style: styles::HAND_CARD,
            onclick: move |event| onclick.call(event),
            span { style: styles::CARD_KIND, "{def.kind:?}" }
            strong { "{def.name}" }
            span { style: styles::SMALL_TEXT, "{def.rules}" }
            span { style: styles::CARD_COST, "{card_cost_label(card)}" }
        }
    }
}

#[component]
fn InteractiveBoardTile(
    row: i32,
    col: i32,
    tile: Option<Tile>,
    players: Vec<PublicPlayer>,
    current_player: Option<PublicPlayer>,
    caravan_row: i32,
    selected: bool,
    view: Signal<Option<GameView>>,
    player_id: Signal<Option<u64>>,
    mut target_row: Signal<i32>,
    mut target_col: Signal<i32>,
) -> Element {
    let move_cost = tile_move_cost(current_player.as_ref(), row, col, caravan_row);
    let can_step = can_step_to(current_player.as_ref(), row, col, caravan_row);

    rsx! {
        BoardTile {
            row,
            col,
            tile,
            players,
            selected,
            move_cost,
            can_step,
            onclick: move |_| async move {
                if can_step {
                    if let Some(id) = *player_id.read()
                        && let Ok(snapshot) = send_game_command(*player_id.read(), ClientCommand::MoveStep {
                            player_id: id,
                            target: Position { row, col },
                        }).await {
                            view.set(Some(snapshot));
                        }
                } else {
                    target_row.set(row);
                    target_col.set(col);
                }
            }
        }
    }
}

#[component]
fn BoardTile(
    row: i32,
    col: i32,
    tile: Option<Tile>,
    players: Vec<PublicPlayer>,
    selected: bool,
    move_cost: Option<i32>,
    can_step: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let class = if selected { "selected" } else { "" };
    rsx! {
        button { class, style: styles::TILE, onclick: move |event| onclick.call(event),
            span { style: styles::COORDS, "{row},{col}" }
            if let Some(tile) = tile {
                strong { "{event_name(&tile.event)}" }
                span { style: styles::SMALL_TEXT, "{event_description(&tile.event)}" }
                if let Some(cost) = move_cost {
                    span { style: if can_step { styles::MOVE_COST_OK } else { styles::MOVE_COST },
                        "{cost} Energy"
                    }
                }
                if tile.sealed_days > 0 {
                    span { style: styles::SEALED, "sealed {tile.sealed_days}" }
                }
            } else {
                strong { "Empty" }
            }
            div { style: styles::OCCUPANTS,
                for player in players {
                    span { style: styles::OCCUPANT, "{player.name.chars().next().unwrap_or('?')}" }
                }
            }
        }
    }
}

fn card_target(card: Card, target_player: Option<u64>, selected_position: Position) -> CardTarget {
    match card_def(card).target {
        TargetKind::None => CardTarget::None,
        TargetKind::Player => target_player
            .map(CardTarget::Player)
            .unwrap_or(CardTarget::None),
        TargetKind::Tile => CardTarget::Tile(selected_position),
    }
}

fn tile_move_cost(
    current_player: Option<&PublicPlayer>,
    row: i32,
    col: i32,
    caravan_row: i32,
) -> Option<i32> {
    current_player.map(|player| movement_cost(&player.pos, &Position { row, col }, caravan_row))
}

fn can_step_to(
    current_player: Option<&PublicPlayer>,
    row: i32,
    col: i32,
    caravan_row: i32,
) -> bool {
    current_player.is_some_and(|player| {
        let distance = (player.pos.row - row).abs() + (player.pos.col - col).abs();
        let cost = movement_cost(&player.pos, &Position { row, col }, caravan_row);
        !player.ready && distance == 1 && player.energy >= cost
    })
}
