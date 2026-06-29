use crate::{
    cards::{TargetKind, card_def},
    model::*,
};
use bevy_ecs::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Clone, Debug)]
struct GameMeta {
    day: u32,
    caravan_row: i32,
    next_id: u64,
    consequence_queue: Vec<EventKind>,
}

#[derive(Resource, Default, Debug)]
struct PendingPlans(HashMap<u64, TurnPlan>);

#[derive(Resource, Debug)]
struct GameLogs(Vec<LogEntry>);

#[derive(Component, Clone, Debug)]
struct PlayerEntity(Player);

#[derive(Component, Clone, Debug)]
struct TileEntity(Tile);

#[derive(Clone, Debug, Default)]
struct GameCache {
    day: u32,
    caravan_row: i32,
    players: Vec<Player>,
    tiles: Vec<Tile>,
    logs: Vec<LogEntry>,
}

pub struct Game {
    world: World,
    cache: GameCache,
}

impl Game {
    pub fn new() -> Self {
        let mut world = World::new();
        world.insert_resource(GameMeta {
            day: 1,
            caravan_row: 0,
            next_id: 1,
            consequence_queue: Vec::new(),
        });
        world.insert_resource(PendingPlans::default());
        world.insert_resource(GameLogs(vec![LogEntry {
            day: 1,
            text: "The caravan lurches into motion. Nobody fully trusts the map.".into(),
            private_to: None,
        }]));
        let mut game = Self {
            world,
            cache: GameCache {
                day: 1,
                caravan_row: 0,
                ..GameCache::default()
            },
        };
        for row in -1..=FORWARD_ROWS {
            game.add_row(row);
        }
        game.refresh_state();
        game
    }

    fn meta(&self) -> &GameMeta {
        self.world.resource::<GameMeta>()
    }

    fn meta_mut(&mut self) -> Mut<'_, GameMeta> {
        self.world.resource_mut::<GameMeta>()
    }

    fn plans(&self) -> &PendingPlans {
        self.world.resource::<PendingPlans>()
    }

    fn plans_mut(&mut self) -> Mut<'_, PendingPlans> {
        self.world.resource_mut::<PendingPlans>()
    }

    fn logs_mut(&mut self) -> Mut<'_, GameLogs> {
        self.world.resource_mut::<GameLogs>()
    }

    pub fn day(&self) -> u32 {
        self.cache.day
    }

    pub fn caravan_row(&self) -> i32 {
        self.cache.caravan_row
    }

    pub fn player_count(&self) -> usize {
        self.cache.players.len()
    }

    pub fn is_ai_player(&self, player_id: u64) -> bool {
        self.player_snapshot(player_id)
            .is_some_and(|player| player.is_ai)
    }

    fn next_id(&mut self) -> u64 {
        let mut meta = self.meta_mut();
        let id = meta.next_id;
        meta.next_id += 1;
        id
    }

    fn with_player_mut<R>(
        &mut self,
        player_id: u64,
        f: impl FnOnce(&mut Player) -> R,
    ) -> Option<R> {
        let mut query = self.world.query::<&mut PlayerEntity>();
        let mut player = query
            .iter_mut(&mut self.world)
            .find(|player| player.0.id == player_id)?;
        Some(f(&mut player.0))
    }

    fn player_snapshot(&self, player_id: u64) -> Option<Player> {
        self.cache
            .players
            .iter()
            .find(|player| player.id == player_id)
            .cloned()
    }

    fn tile_snapshot(&self, pos: &Position) -> Option<Tile> {
        self.cache
            .tiles
            .iter()
            .find(|tile| tile.pos == *pos)
            .cloned()
    }

    fn tile_mut_at(&mut self, pos: &Position) -> Option<Mut<'_, TileEntity>> {
        let mut query = self.world.query::<&mut TileEntity>();
        query
            .iter_mut(&mut self.world)
            .find(|tile| tile.0.pos == *pos)
    }

    fn player_name(&self, player_id: u64) -> String {
        self.player_snapshot(player_id)
            .map(|player| player.name)
            .unwrap_or_else(|| "Unknown wizard".into())
    }

    fn public_log(&mut self, text: String) {
        let day = self.day();
        let entry = LogEntry {
            day,
            text,
            private_to: None,
        };
        self.logs_mut().0.push(entry.clone());
        self.cache.logs.push(entry);
    }

    pub fn private_log(&mut self, player_id: u64, text: String) {
        let day = self.day();
        let entry = LogEntry {
            day,
            text,
            private_to: Some(player_id),
        };
        self.logs_mut().0.push(entry.clone());
        self.cache.logs.push(entry);
    }

    pub fn add_player(&mut self, name: String, is_ai: bool) -> u64 {
        let id = self.next_id();
        let col = BOARD_WIDTH / 2;
        let caravan_row = self.caravan_row();
        self.world.spawn(PlayerEntity(Player {
            id,
            name: name.clone(),
            is_ai,
            pos: Position {
                row: caravan_row,
                col,
            },
            energy: MAX_ENERGY,
            gold: 2,
            food: 3,
            glory: 0,
            reagents: Reagents {
                ash: 1,
                mooncap: 1,
                ichor: 1,
                quartz: 1,
            },
            hand: starting_hand(id),
            statuses: Vec::new(),
            public_gear: Vec::new(),
        }));
        self.public_log(format!("{name} joins the quest."));
        self.refresh_state();
        id
    }

    pub fn view_for(&self, viewer_id: Option<u64>) -> GameView {
        let plans = self.plans();
        let mut players = self
            .cache
            .players
            .iter()
            .map(|player| PublicPlayer {
                id: player.id,
                name: player.name.clone(),
                is_ai: player.is_ai,
                pos: player.pos.clone(),
                energy: player.energy,
                gold: player.gold,
                food: player.food,
                glory: player.glory,
                statuses: player.statuses.clone(),
                public_gear: player.public_gear.clone(),
                ready: plans.0.contains_key(&player.id),
            })
            .collect::<Vec<_>>();
        players.sort_by_key(|player| player.id);
        let own = viewer_id.and_then(|id| {
            self.player_snapshot(id).map(|player| OwnPlayer {
                id,
                hand: player.hand,
                reagents: player.reagents,
            })
        });
        let mut winners = Vec::new();
        if self.cache.day > QUEST_DAYS {
            if let Some(best) = players.iter().map(|player| player.glory).max() {
                winners = players
                    .iter()
                    .filter(|player| player.glory == best)
                    .cloned()
                    .collect();
            }
        }
        GameView {
            viewer_id,
            day: self.cache.day,
            caravan_row: self.cache.caravan_row,
            width: BOARD_WIDTH,
            players,
            tiles: self.cache.tiles.clone(),
            own,
            logs: self
                .cache
                .logs
                .iter()
                .filter(|entry| entry.private_to.is_none() || entry.private_to == viewer_id)
                .cloned()
                .rev()
                .take(80)
                .collect(),
            winners,
        }
    }

    pub fn submit_plan(&mut self, player_id: u64, plan: TurnPlan) -> Result<(), String> {
        if self.cache.day > QUEST_DAYS {
            return Err("The quest is over.".into());
        }
        if self.player_snapshot(player_id).is_none() {
            return Err("Unknown player.".into());
        }
        self.plans_mut().0.insert(player_id, plan);
        if self.plans().0.len() == self.player_count() && self.player_count() > 0 {
            self.resolve_day();
        }
        Ok(())
    }

    pub fn next_unready_ai(&self) -> Option<u64> {
        self.cache
            .players
            .iter()
            .find(|player| player.is_ai && !self.plans().0.contains_key(&player.id))
            .map(|player| player.id)
    }

    pub fn send_chat(&mut self, player_id: u64, message: String) -> Result<(), String> {
        let message = message.trim();
        if message.is_empty() {
            return Ok(());
        }
        let name = self.player_name(player_id);
        self.public_log(format!("{name}: {message}"));
        Ok(())
    }

    pub fn move_step(&mut self, player_id: u64, target: Position) -> Result<(), String> {
        if self.plans().0.contains_key(&player_id) {
            return Err("You already ended your turn for this day.".into());
        }
        let Some(start) = self.player_snapshot(player_id).map(|player| player.pos) else {
            return Err("Unknown player.".into());
        };
        let target = Position {
            row: target
                .row
                .clamp(self.caravan_row() - 1, self.caravan_row() + FORWARD_ROWS),
            col: target.col.clamp(0, BOARD_WIDTH - 1),
        };
        let distance = (target.row - start.row).abs() + (target.col - start.col).abs();
        if distance != 1 {
            return Err("Move one step at a time.".into());
        }
        let cost = movement_cost(&start, &target, self.caravan_row());
        let mut moved = false;
        self.with_player_mut(player_id, |player| {
            if player.energy >= cost {
                player.energy -= cost;
                player.pos = target.clone();
                moved = true;
            }
        });
        if !moved {
            return Err(format!("Not enough energy. This step costs {cost}."));
        }
        let name = self.player_name(player_id);
        self.public_log(format!(
            "{name} moves to row {}, column {} for {cost} Energy.",
            target.row, target.col
        ));
        self.refresh_state();
        Ok(())
    }

    fn add_row(&mut self, row: i32) {
        for col in 0..BOARD_WIDTH {
            let event = {
                let mut meta = self.meta_mut();
                meta.consequence_queue
                    .pop()
                    .unwrap_or_else(|| event_for(meta.day, row, col))
            };
            let id = self.next_id();
            self.world.spawn(TileEntity(Tile {
                id,
                pos: Position { row, col },
                event,
                sealed_days: 0,
                completed_by: None,
            }));
        }
    }

    fn resolve_day(&mut self) {
        let plans = std::mem::take(&mut self.plans_mut().0);
        self.public_log(format!("Day {} plans resolve.", self.day()));
        for player_id in self.ordered_player_ids() {
            let Some(plan) = plans.get(&player_id).cloned() else {
                continue;
            };
            self.resolve_card(player_id, plan.card_action.clone());
            self.resolve_movement_and_event(player_id, plan);
        }
        self.resolve_upkeep();
        self.advance_caravan();
        self.meta_mut().day += 1;
        if self.day() > QUEST_DAYS {
            self.public_log("The caravan reaches the final camp. Glory is counted.".into());
        } else {
            self.public_log(format!("Day {} begins.", self.day()));
        }
        self.refresh_state();
    }

    fn resolve_card(&mut self, player_id: u64, action: CardAction) {
        match action {
            CardAction::None => {}
            CardAction::Scrap { card } => {
                let mut log = None;
                let _ = self.with_player_mut(player_id, |player| {
                    if remove_card(&mut player.hand, &card) {
                        player.gold += 1;
                        player.reagents.ash += 1;
                        log = Some(format!(
                            "{} scraps {} for supplies.",
                            player.name,
                            card_name(&card)
                        ));
                    }
                });
                if let Some(log) = log {
                    self.public_log(log);
                }
            }
            CardAction::PlayCard {
                card,
                target,
                question,
            } => self.play_card(player_id, card, target, question),
        }
    }

    fn play_card(
        &mut self,
        player_id: u64,
        card: Card,
        target: CardTarget,
        question: Option<String>,
    ) {
        let valid_target = matches!(
            (card_def(card).target, &target),
            (TargetKind::None, CardTarget::None)
                | (TargetKind::Player, CardTarget::Player(_))
                | (TargetKind::Tile, CardTarget::Tile(_))
        );
        if !valid_target {
            self.public_log(format!(
                "{} tries to play {} without a valid target.",
                self.player_name(player_id),
                card_def(card).name
            ));
            return;
        }

        if !self.pay_card_cost(player_id, card) {
            return;
        }

        match card {
            Card::PoisonedBreakfast => {
                let CardTarget::Player(target_player_id) = target else {
                    return;
                };
                let mut target_name = None;
                let _ = self.with_player_mut(target_player_id, |target| {
                    target.statuses.push(Status::FoodSickness { days: 1 });
                    target_name = Some(target.name.clone());
                });
                self.private_log(
                    target_player_id,
                    "You wake up with Food Sickness. Someone tampered with breakfast.".into(),
                );
                if let Some(name) = target_name {
                    self.public_log(format!("{name} looks ill after breakfast."));
                }
            }
            Card::SpellOfTruth => {
                let CardTarget::Player(target_player_id) = target else {
                    return;
                };
                let caster = self.player_name(player_id);
                let target = self.player_name(target_player_id);
                let question = question
                    .filter(|question| !question.trim().is_empty())
                    .unwrap_or_else(|| "What are you planning?".into());
                self.public_log(format!(
                    "{caster} casts Spell of Truth on {target}: \"{question}\""
                ));
            }
            Card::StaffOfSparkAmplification => {
                let mut log = None;
                let _ = self.with_player_mut(player_id, |player| {
                    player.public_gear.push(card_def(card).name.into());
                    log = Some(format!("{} equips {}.", player.name, card_def(card).name));
                });
                if let Some(log) = log {
                    self.public_log(log);
                }
            }
            Card::HatOfModestProsperity => {
                let mut log = None;
                let _ = self.with_player_mut(player_id, |player| {
                    player.public_gear.push(card_def(card).name.into());
                    log = Some(format!("{} equips {}.", player.name, card_def(card).name));
                });
                if let Some(log) = log {
                    self.public_log(log);
                }
            }
            Card::SuspiciouslyHelpfulMap => {
                let mut log = None;
                let _ = self.with_player_mut(player_id, |player| {
                    player.reagents.quartz += 1;
                    log = Some(format!(
                        "{} consults a suspiciously helpful map.",
                        player.name
                    ));
                });
                if let Some(log) = log {
                    self.public_log(log);
                }
            }
            Card::AlarmWard => {
                let mut log = None;
                let _ = self.with_player_mut(player_id, |player| {
                    player.statuses.push(Status::AlarmWard);
                    log = Some(format!("{} prepares a faintly humming ward.", player.name));
                });
                if let Some(log) = log {
                    self.public_log(log);
                }
            }
            Card::EventSeal => {
                let CardTarget::Tile(pos) = target else {
                    return;
                };
                if let Some(mut tile) = self.tile_mut_at(&pos) {
                    tile.0.sealed_days = 2;
                    self.public_log(format!(
                        "A tile at row {}, column {} is sealed.",
                        pos.row, pos.col
                    ));
                }
            }
        }
    }

    fn pay_card_cost(&mut self, player_id: u64, card: Card) -> bool {
        let def = card_def(card);
        let Some(player) = self.player_snapshot(player_id) else {
            return false;
        };
        if !player.hand.contains(&card)
            || player.energy < def.cost.energy
            || player.gold < def.cost.gold
            || player.food < def.cost.food
            || !player.reagents.can_pay(&def.cost.reagents)
        {
            return false;
        }
        self.with_player_mut(player_id, |player| {
            player.energy -= def.cost.energy;
            player.gold -= def.cost.gold;
            player.food -= def.cost.food;
            player.reagents.spend(&def.cost.reagents);
            if def.one_use || matches!(def.target, TargetKind::None) {
                remove_card(&mut player.hand, &card);
            }
        });
        true
    }

    fn resolve_movement_and_event(&mut self, player_id: u64, plan: TurnPlan) {
        let Some(snapshot) = self.player_snapshot(player_id) else {
            return;
        };
        let start = snapshot.pos.clone();
        let target = Position {
            row: plan
                .target
                .row
                .clamp(self.caravan_row() - 1, self.caravan_row() + FORWARD_ROWS),
            col: plan.target.col.clamp(0, BOARD_WIDTH - 1),
        };
        let cost = movement_cost(&start, &target, self.caravan_row());
        let player_name = snapshot.name.clone();
        if snapshot.energy < cost {
            self.public_log(format!(
                "{player_name} is too exhausted to reach row {}, column {}.",
                target.row, target.col
            ));
            return;
        }
        let _ = self.with_player_mut(player_id, |player| {
            player.energy -= cost;
            player.pos = target.clone();
        });
        if !plan.chat.trim().is_empty() {
            self.public_log(format!("{player_name}: {}", plan.chat.trim()));
        }
        if cost > 0 {
            self.public_log(format!(
                "{player_name} moves to row {}, column {} for {cost} energy.",
                target.row, target.col
            ));
        }

        match plan.event_action {
            EventAction::Attempt => self.attempt_event(player_id, target),
            EventAction::Rest => {
                let _ = self.with_player_mut(player_id, |player| {
                    player.energy = (player.energy + 3).min(MAX_ENERGY);
                });
                self.public_log(format!("{player_name} rests instead of pushing luck."));
            }
            EventAction::Ignore => {}
        }
    }

    fn attempt_event(&mut self, player_id: u64, pos: Position) {
        let Some(tile_snapshot) = self.tile_snapshot(&pos) else {
            self.public_log(format!(
                "{} finds no available event there.",
                self.player_name(player_id)
            ));
            return;
        };
        if tile_snapshot.completed_by.is_some() {
            self.public_log(format!(
                "{} finds no available event there.",
                self.player_name(player_id)
            ));
            return;
        }
        if tile_snapshot.sealed_days > 0 {
            self.public_log(format!(
                "{} reaches a sealed event and cannot enter.",
                self.player_name(player_id)
            ));
            return;
        }

        let event = tile_snapshot.event.clone();
        let day = self.day();
        let player_name = self.player_name(player_id);
        if let Some(mut tile) = self.tile_mut_at(&pos) {
            tile.0.completed_by = Some(player_id);
        }
        let mut public_logs = Vec::new();
        let mut add_witch = false;
        let mut add_beggar = false;

        let _ = self.with_player_mut(player_id, |player| match event {
            EventKind::Monster => {
                player.energy -= 2;
                player.glory += 3;
                player.reagents.ichor += 1;
                public_logs.push(format!("{player_name} defeats a monster for 3 Glory."));
            }
            EventKind::Hunt => {
                player.energy -= 1;
                player.food += 3;
                player.glory += 1;
                public_logs.push(format!(
                    "{player_name} returns from a hunt with Food and 1 Glory."
                ));
            }
            EventKind::GuardedTreasure => {
                player.gold += 3;
                player.glory += 2;
                player.hand.push(draw_card(player.id + day as u64));
                public_logs.push(format!("{player_name} cracks a guarded treasure."));
            }
            EventKind::CursedShrine => {
                player.glory += 4;
                player.statuses.push(Status::MildCurse { days: 3 });
                public_logs.push(format!("{player_name} takes shrine power and a curse."));
            }
            EventKind::Market => {
                player.gold += 1;
                player.hand.push(draw_card(player.id * 7 + day as u64));
                add_witch = player_id % 2 == 0;
                public_logs.push(format!("{player_name} trades at a market caravan."));
            }
            EventKind::Library => {
                player.reagents.quartz += 1;
                player.hand.push(Card::SpellOfTruth);
                public_logs.push(format!("{player_name} finds a rare spell in a library."));
            }
            EventKind::Camp => {
                player.energy = MAX_ENERGY;
                player.food += 1;
                add_beggar = true;
                public_logs.push(format!("{player_name} recovers at camp."));
            }
            EventKind::Trap => {
                player.energy -= 3;
                player.glory += 1;
                public_logs.push(format!("{player_name} survives a trap and earns 1 Glory."));
            }
            EventKind::Shortcut => {
                player.pos.row += 1;
                player.glory += 1;
                public_logs.push(format!("{player_name} takes a reckless shortcut."));
            }
            EventKind::WitchRevenge => {
                player.statuses.push(Status::MildCurse { days: 4 });
                player.glory += 1;
                public_logs.push(format!("{player_name} suffers a witch's delayed revenge."));
            }
            EventKind::BeggarThanks => {
                player.food += 2;
                player.gold += 2;
                player.glory += 2;
                public_logs.push(format!("{player_name} is repaid by old kindness."));
            }
        });

        for log in public_logs {
            self.public_log(log);
        }
        if add_witch {
            self.meta_mut()
                .consequence_queue
                .push(EventKind::WitchRevenge);
            self.public_log("A future consequence slips into the road ahead.".into());
        }
        if add_beggar {
            self.meta_mut()
                .consequence_queue
                .push(EventKind::BeggarThanks);
        }
        self.refresh_state();
    }

    fn resolve_upkeep(&mut self) {
        let mut logs = Vec::new();
        for player_id in self.ordered_player_ids() {
            let _ = self.with_player_mut(player_id, |player| {
                let sick = player
                    .statuses
                    .iter()
                    .any(|status| matches!(status, Status::FoodSickness { .. }));
                let recovery = if sick { 2 } else { 4 };
                player.energy = (player.energy + recovery).min(MAX_ENERGY);
                for status in &mut player.statuses {
                    match status {
                        Status::FoodSickness { days } | Status::MildCurse { days } => {
                            *days = days.saturating_sub(1);
                        }
                        Status::AlarmWard => {}
                    }
                }
                player.statuses.retain(|status| match status {
                    Status::FoodSickness { days } | Status::MildCurse { days } => *days > 0,
                    Status::AlarmWard => true,
                });
                if player
                    .public_gear
                    .iter()
                    .any(|gear| gear == "Hat of Modest Prosperity")
                {
                    player.gold += 1;
                    logs.push(format!("{}'s hat produces 1 Gold.", player.name));
                }
            });
        }
        for log in logs {
            self.public_log(log);
        }
        let mut query = self.world.query::<&mut TileEntity>();
        for mut tile in query.iter_mut(&mut self.world) {
            tile.0.sealed_days = tile.0.sealed_days.saturating_sub(1);
        }
        self.refresh_state();
    }

    fn advance_caravan(&mut self) {
        self.meta_mut().caravan_row += 2;
        let min_row = self.caravan_row() - 1;
        let max_row = self.caravan_row() + FORWARD_ROWS;
        let to_remove = self
            .world
            .query::<(Entity, &TileEntity)>()
            .iter(&self.world)
            .filter(|(_, tile)| tile.0.pos.row < min_row || tile.0.completed_by.is_some())
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>();
        for entity in to_remove {
            let _ = self.world.despawn(entity);
        }
        for row in (max_row - 1)..=max_row {
            let has_row = self
                .world
                .query::<&TileEntity>()
                .iter(&self.world)
                .any(|tile| tile.0.pos.row == row);
            if !has_row {
                self.add_row(row);
            }
        }
        let mut player_query = self.world.query::<&mut PlayerEntity>();
        for mut player in player_query.iter_mut(&mut self.world) {
            if player.0.pos.row < min_row {
                player.0.pos.row = min_row;
                player.0.energy = (player.0.energy - 1).max(0);
            }
        }
        self.refresh_state();
    }

    fn ordered_player_ids(&self) -> Vec<u64> {
        let mut ids = self
            .cache
            .players
            .iter()
            .map(|player| player.id)
            .collect::<Vec<_>>();
        ids.sort_unstable();
        ids
    }

    fn refresh_state(&mut self) {
        let (day, caravan_row) = {
            let meta = self.meta();
            (meta.day, meta.caravan_row)
        };
        self.cache.day = day;
        self.cache.caravan_row = caravan_row;

        let mut players = self
            .world
            .query::<&PlayerEntity>()
            .iter(&self.world)
            .map(|player| player.0.clone())
            .collect::<Vec<_>>();
        players.sort_by_key(|player| player.id);
        self.cache.players = players;

        self.cache.tiles = self
            .world
            .query::<&TileEntity>()
            .iter(&self.world)
            .map(|tile| tile.0.clone())
            .collect::<Vec<_>>();
    }
}

fn starting_hand(seed: u64) -> Vec<Card> {
    vec![
        draw_card(seed),
        Card::PoisonedBreakfast,
        Card::AlarmWard,
        Card::EventSeal,
    ]
}

fn draw_card(seed: u64) -> Card {
    match seed % 7 {
        0 => Card::PoisonedBreakfast,
        1 => Card::SpellOfTruth,
        2 => Card::StaffOfSparkAmplification,
        3 => Card::HatOfModestProsperity,
        4 => Card::SuspiciouslyHelpfulMap,
        5 => Card::AlarmWard,
        _ => Card::EventSeal,
    }
}

fn event_for(day: u32, row: i32, col: i32) -> EventKind {
    match ((day as i32 + row * 3 + col * 5).rem_euclid(9)) as u8 {
        0 => EventKind::Monster,
        1 => EventKind::Hunt,
        2 => EventKind::GuardedTreasure,
        3 => EventKind::CursedShrine,
        4 => EventKind::Market,
        5 => EventKind::Library,
        6 => EventKind::Camp,
        7 => EventKind::Trap,
        _ => EventKind::Shortcut,
    }
}
