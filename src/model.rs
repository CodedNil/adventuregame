#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const BOARD_WIDTH: i32 = 7;
pub const FORWARD_ROWS: i32 = 4;
pub const QUEST_DAYS: u32 = 20;
pub const MAX_ENERGY: i32 = 10;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub row: i32,
    pub col: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Tile {
    pub id: u64,
    pub pos: Position,
    pub event: EventKind,
    pub sealed_days: u8,
    pub completed_by: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EventKind {
    Monster,
    Hunt,
    GuardedTreasure,
    CursedShrine,
    Market,
    Library,
    Camp,
    Trap,
    Shortcut,
    WitchRevenge,
    BeggarThanks,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub is_ai: bool,
    pub pos: Position,
    pub energy: i32,
    pub gold: i32,
    pub food: i32,
    pub glory: i32,
    pub reagents: Reagents,
    pub hand: Vec<Card>,
    pub statuses: Vec<Status>,
    pub public_gear: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Reagents {
    pub ash: i32,
    pub mooncap: i32,
    pub ichor: i32,
    pub quartz: i32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Serialize, Deserialize, PartialEq)]
pub enum Card {
    PoisonedBreakfast,
    SpellOfTruth,
    StaffOfSparkAmplification,
    HatOfModestProsperity,
    SuspiciouslyHelpfulMap,
    AlarmWard,
    EventSeal,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Status {
    FoodSickness { days: u8 },
    AlarmWard,
    MildCurse { days: u8 },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    pub day: u32,
    pub text: String,
    pub private_to: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PublicPlayer {
    pub id: u64,
    pub name: String,
    pub is_ai: bool,
    pub pos: Position,
    pub energy: i32,
    pub gold: i32,
    pub food: i32,
    pub glory: i32,
    pub statuses: Vec<Status>,
    pub public_gear: Vec<String>,
    pub ready: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OwnPlayer {
    pub id: u64,
    pub hand: Vec<Card>,
    pub reagents: Reagents,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GameView {
    pub viewer_id: Option<u64>,
    pub day: u32,
    pub caravan_row: i32,
    pub width: i32,
    pub players: Vec<PublicPlayer>,
    pub tiles: Vec<Tile>,
    pub own: Option<OwnPlayer>,
    pub logs: Vec<LogEntry>,
    pub winners: Vec<PublicPlayer>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TurnPlan {
    pub target: Position,
    pub event_action: EventAction,
    pub card_action: CardAction,
    pub chat: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EventAction {
    Attempt,
    Rest,
    Ignore,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CardAction {
    None,
    Scrap {
        card: Card,
    },
    PlayCard {
        card: Card,
        target: CardTarget,
        question: Option<String>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CardTarget {
    None,
    Player(u64),
    Tile(Position),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientCommand {
    JoinHuman { name: String },
    AddAi,
    SendChat { player_id: u64, message: String },
    MoveStep { player_id: u64, target: Position },
    SubmitPlan { player_id: u64, plan: TurnPlan },
    AskAiTurn { player_id: u64 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    Snapshot(GameView),
    Joined { player_id: u64 },
    Error(String),
}

pub fn card_name(card: &Card) -> &'static str {
    crate::cards::card_def(*card).name
}

pub fn event_name(event: &EventKind) -> &'static str {
    match event {
        EventKind::Monster => "Monster",
        EventKind::Hunt => "Hunt",
        EventKind::GuardedTreasure => "Guarded Treasure",
        EventKind::CursedShrine => "Cursed Shrine",
        EventKind::Market => "Market",
        EventKind::Library => "Library",
        EventKind::Camp => "Camp",
        EventKind::Trap => "Trap",
        EventKind::Shortcut => "Shortcut",
        EventKind::WitchRevenge => "Witch Revenge",
        EventKind::BeggarThanks => "Beggar Thanks",
    }
}

pub fn event_description(event: &EventKind) -> &'static str {
    match event {
        EventKind::Monster => "Fight for Glory and monster reagents. Costs extra energy.",
        EventKind::Hunt => "Spend effort to gather Food and a little Glory.",
        EventKind::GuardedTreasure => "Break into a protected cache for Gold, Glory, and a card.",
        EventKind::CursedShrine => "Take a strong Glory reward and suffer a short curse.",
        EventKind::Market => "Trade for Gold and draw a card. Some choices may echo later.",
        EventKind::Library => "Gain Quartz and a rare truth spell.",
        EventKind::Camp => "Recover fully and gain Food. Kindness may return later.",
        EventKind::Trap => "Survive a hazard. Painful, but worth a little Glory.",
        EventKind::Shortcut => "Leap ahead and gain Glory, usually at positional risk.",
        EventKind::WitchRevenge => "A delayed curse from somebody's earlier mistake.",
        EventKind::BeggarThanks => "A future reward from earlier generosity.",
    }
}

pub fn movement_cost(start: &Position, target: &Position, caravan_row: i32) -> i32 {
    let distance = (target.row - start.row).abs() + (target.col - start.col).abs();
    let ahead_tax = (target.row - caravan_row - 1).max(0);
    (distance + ahead_tax).max(0)
}

pub fn remove_card(hand: &mut Vec<Card>, card: &Card) -> bool {
    if let Some(index) = hand.iter().position(|candidate| candidate == card) {
        hand.remove(index);
        true
    } else {
        false
    }
}
