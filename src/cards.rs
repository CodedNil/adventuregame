use crate::model::{Card, Reagents};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CardKind {
    Scheme,
    Spell,
    Equipment,
    Item,
    Ward,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TargetKind {
    None,
    Player,
    Tile,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CardCost {
    pub energy: i32,
    pub gold: i32,
    pub food: i32,
    pub reagents: Reagents,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardDef {
    pub id: Card,
    pub name: &'static str,
    pub kind: CardKind,
    pub target: TargetKind,
    pub one_use: bool,
    pub rare: bool,
    pub cost: CardCost,
    pub rules: &'static str,
}

pub const CARD_DEFS: &[CardDef] = &[
    CardDef {
        id: Card::PoisonedBreakfast,
        name: "Poisoned Breakfast",
        kind: CardKind::Scheme,
        target: TargetKind::Player,
        one_use: true,
        rare: false,
        cost: CardCost {
            food: 1,
            reagents: Reagents {
                mooncap: 1,
                ..Reagents::ZERO
            },
            ..CardCost::ZERO
        },
        rules: "Target wakes next morning with Food Sickness. Anonymous.",
    },
    CardDef {
        id: Card::SpellOfTruth,
        name: "Spell of Truth",
        kind: CardKind::Spell,
        target: TargetKind::Player,
        one_use: true,
        rare: true,
        cost: CardCost {
            reagents: Reagents {
                quartz: 1,
                ..Reagents::ZERO
            },
            ..CardCost::ZERO
        },
        rules: "Ask one public question. Target must answer truthfully over voice.",
    },
    CardDef {
        id: Card::StaffOfSparkAmplification,
        name: "Staff of Spark Amplification",
        kind: CardKind::Equipment,
        target: TargetKind::None,
        one_use: false,
        rare: false,
        cost: CardCost::ZERO,
        rules: "Equip. Lightning or attack spells are stronger in monster encounters.",
    },
    CardDef {
        id: Card::HatOfModestProsperity,
        name: "Hat of Modest Prosperity",
        kind: CardKind::Equipment,
        target: TargetKind::None,
        one_use: false,
        rare: false,
        cost: CardCost::ZERO,
        rules: "Equip. Gain 1 Gold during upkeep.",
    },
    CardDef {
        id: Card::SuspiciouslyHelpfulMap,
        name: "Suspiciously Helpful Map",
        kind: CardKind::Item,
        target: TargetKind::None,
        one_use: true,
        rare: false,
        cost: CardCost::ZERO,
        rules: "Peek at the road ahead. Prototype: gain 1 Quartz.",
    },
    CardDef {
        id: Card::AlarmWard,
        name: "Alarm Ward",
        kind: CardKind::Ward,
        target: TargetKind::None,
        one_use: true,
        rare: false,
        cost: CardCost {
            reagents: Reagents {
                quartz: 1,
                ..Reagents::ZERO
            },
            ..CardCost::ZERO
        },
        rules: "Prepare a camp ward against poison, theft, curses, and tampering.",
    },
    CardDef {
        id: Card::EventSeal,
        name: "Event Seal",
        kind: CardKind::Spell,
        target: TargetKind::Tile,
        one_use: true,
        rare: false,
        cost: CardCost {
            reagents: Reagents {
                ash: 1,
                quartz: 1,
                ..Reagents::ZERO
            },
            ..CardCost::ZERO
        },
        rules: "Seal an event tile for 2 days so nobody can complete it.",
    },
];

impl Reagents {
    pub const ZERO: Self = Self {
        ash: 0,
        mooncap: 0,
        ichor: 0,
        quartz: 0,
    };

    #[allow(dead_code)]
    pub fn can_pay(&self, cost: &Self) -> bool {
        self.ash >= cost.ash
            && self.mooncap >= cost.mooncap
            && self.ichor >= cost.ichor
            && self.quartz >= cost.quartz
    }

    #[allow(dead_code)]
    pub fn spend(&mut self, cost: &Self) {
        self.ash -= cost.ash;
        self.mooncap -= cost.mooncap;
        self.ichor -= cost.ichor;
        self.quartz -= cost.quartz;
    }
}

impl CardCost {
    pub const ZERO: Self = Self {
        energy: 0,
        gold: 0,
        food: 0,
        reagents: Reagents::ZERO,
    };
}

pub fn card_def(card: Card) -> &'static CardDef {
    CARD_DEFS
        .iter()
        .find(|def| def.id == card)
        .expect("all card enum variants must have card definitions")
}

pub fn card_cost_label(card: Card) -> String {
    let cost = card_def(card).cost;
    let mut parts = Vec::new();
    if cost.energy > 0 {
        parts.push(format!("{} Energy", cost.energy));
    }
    if cost.gold > 0 {
        parts.push(format!("{} Gold", cost.gold));
    }
    if cost.food > 0 {
        parts.push(format!("{} Food", cost.food));
    }
    if cost.reagents.ash > 0 {
        parts.push(format!("{} Ash", cost.reagents.ash));
    }
    if cost.reagents.mooncap > 0 {
        parts.push(format!("{} Mooncap", cost.reagents.mooncap));
    }
    if cost.reagents.ichor > 0 {
        parts.push(format!("{} Ichor", cost.reagents.ichor));
    }
    if cost.reagents.quartz > 0 {
        parts.push(format!("{} Quartz", cost.reagents.quartz));
    }
    if parts.is_empty() {
        "Free".into()
    } else {
        parts.join(", ")
    }
}
