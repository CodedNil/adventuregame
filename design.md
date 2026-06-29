# The Unreliable Fellowship Design

## Pitch

A 3-8 player web board/card game about bumbling wizards on a dangerous quest. Players travel with a caravan, race across a moving event map, collect gear and spell reagents, cooperate when useful, and betray when profitable. One player wins.

Reference vibes: 7 Wonders pace, Werewolf/Mafia suspicion, Town of Salem accusation energy, but with RPG inventory and a shared travelling map.

## Design Goals

- Web-first Dioxus fullstack game.
- 3-8 players.
- About 30 minutes for a standard game.
- Simultaneous planning so players are not waiting through long turns.
- Board movement, event denial, cooperation, and sabotage all matter.
- Cards stay simple because the game will have many of them.
- No player elimination.
- Social play is expected over third-party voice chat.

## Core Loop

Each turn is one day.

1. Players secretly plan movement, event actions, spells, trades, traps, wards, and sabotage.
2. Everyone commits.
3. The server resolves movement, events, cards, cooperation, sabotage, and rewards.
4. Completed events are removed.
5. Night/upkeep effects resolve.
6. The caravan moves forward and new events are added.

Default game length: 20 days.

## Travelling Map

Players move around a conveyor-style event map. The caravan moves forward automatically.

Map rules:

- Players have board positions.
- Each tile contains one event.
- Completed events disappear.
- Some events are missed because the map moves faster than players can clear it.
- The caravan moves forward 2 rows per day.
- There is 1 row behind the caravan, the caravan's current row, and several rows ahead.
- When the caravan moves, old rows fall off and 2 new rows are added at the front.

Recommended starting shape:

- Width: 6 columns.
- Depth: 1 row behind caravan + caravan row + forward rows.
- For 6 players, 6-wide with 2 new rows per day means 12 new events per day, or 2 new events per player.

Example:

```text
Future       [ ] [ ] [ ] [ ] [ ] [ ]
             [ ] [ ] [ ] [ ] [ ] [ ]
             [ ] [ ] [ ] [ ] [ ] [ ]
Caravan      [ ] [ ] [ ] [ ] [ ] [ ]
Behind       [ ] [ ] [ ] [ ] [ ] [ ]
```

Map size should remain configurable. Smaller games may use fewer columns; larger games may need more width or depth.

## Movement

Movement costs energy.

- Moving sideways is allowed.
- Moving forward is allowed.
- Moving two or more rows forward in one day is allowed if the player can pay.
- Being far ahead of the caravan increases movement or survival costs.
- Staying near the caravan is cheaper and safer.
- Rushing ahead lets players claim strong events first, but leaves them low on energy.

The caravan is the cost anchor. The farther ahead a wizard is, the more expensive it is to keep pushing.

Open movement questions:

- Exact movement cost formula.
- Daily energy recovery.
- Whether players have only a current tile or both a camp position and current tile.
- What happens when multiple players reach the same event at the same time.

## Event Tiles

Events are the main board content.

Event types:

- **Monster Encounter**: Fight for Glory, treasure, or materials.
- **Hunt**: Spend energy for Food, reagents, trophies, or Glory.
- **Guarded Treasure**: Reward protected by combat, stealth, bribes, or magic.
- **Cursed Shrine**: Strong reward with a curse or cost.
- **Market Caravan**: Buy, sell, trade, upgrade, or scrap cards.
- **Mystic Library**: Draw spells, reveal information, or gain rare reagents.
- **Tavern/Camp**: Recover, gossip, trade, or make deals.
- **Trap**: Punishes careless movement unless disarmed or redirected.
- **Shortcut**: Changes position, movement cost, or future map generation.
- **Quest Event**: Advances shared or personal quest progress.

Event denial matters:

- A player can complete a valuable event before others reach it.
- Spells can trap or seal an event for 1-2 days.
- Some cards can peek at hidden event details.
- Players cannot complete everything before the caravan moves on.

## Consequence Events

Some event choices insert future cards into the event deck.

Negative examples:

- Steal from a witch selling potions. A future mystery event becomes the witch's revenge and curses whoever finds it.
- Shoo a bear cub away from a corpse to steal a treasure bag. A future event becomes mother bear revenge.

Positive examples:

- Give Gold or Food to beggars. A later event may reward the party with help, shelter, information, or supplies.

Consequence cards can be inserted randomly into the future event deck, delayed by a few days, or placed into a special consequence queue mixed into new rows.

## Players

Public state:

- Wizard name/avatar.
- Board position.
- Glory or rough standing.
- Energy.
- Gold and Food.
- Public equipment.
- Visible statuses.
- Ready/locked state.

Private state:

- Hand cards.
- Reagents.
- Hidden statuses.
- Prepared spells.
- Anonymous sabotage.
- Camp defenses.
- Secret objectives, if used.

## Resources

Core resources:

- **Energy**: Movement, events, exertion. Recovers each day.
- **Gold**: Buying, bribing, trading, event choices.
- **Food**: Travel, recovery, feasts, poisoning, camp/night effects.
- **Glory**: Main score resource.

Spell reagents:

- **Ash**: Fire, destruction, curses, decay, dangerous shortcuts.
- **Mooncap**: Illusion, sleep, poison, dreams, secrecy, bargains.
- **Ichor**: Life, beasts, blood magic, monsters, healing, mutation.
- **Quartz**: Wards, truth, lightning, locks, protection.

There is no mana and no lore resource. Spells cost reagent combinations.

Simple reagent rules:

- Minor spell: 1 reagent.
- Strong spell: 2 reagents.
- Very strong spell: reagents plus another cost, such as energy, Food, Gold, or sacrificing a card.

## Cards

Cards are the player's inventory.

Card types:

- **Spells**: One-use or reusable magical effects.
- **Items**: Potions, traps, scrolls, keys, charms, tools.
- **Equipment**: Staffs, hats, robes, rings, boots, familiars.
- **Wards**: Prepared defenses, usually for camp/night.
- **Curses**: Negative cards from events or players.
- **Schemes**: Hidden sabotage or social manipulation.
- **Companions**: Temporary helpers.

Cards can be drawn, played, equipped, prepared, scrapped, traded, stolen, copied, cursed, upgraded, or destroyed.

Trading cards and resources is allowed. Trading cards reveals information, because other players may learn what you have.

Scrapping unwanted cards gives resources so bad draws still have value.

## Example Cards

### Poisoned Breakfast

Type: Scheme or Spell  
Cost: Mooncap, Food, or poison item  
Target: One player  
Visibility: Anonymous  
Effect: Play during the day. The target wakes up next morning with Food Sickness, reducing energy recovery or movement efficiency for one day.

### Spell of Truth

Type: Rare one-use Spell  
Cost: Quartz  
Target: One player  
Visibility: Public  
Effect: Ask one question. The target must answer truthfully. This is socially enforced over voice chat.

### Staff of Spark Amplification

Type: Equipment  
Effect: Lightning or attack spells cost 1 fewer Quartz, or give +1 power during monster encounters.

### Hat of Modest Prosperity

Type: Equipment  
Effect: Gain 1 Gold, 1 Food, or 1 minor reagent during upkeep.

### Suspiciously Helpful Map

Type: Item  
Effect: Peek at the next generated rows or replace one future tile. All players know the map was used.

### Alarm Ward

Type: Ward  
Cost: Quartz  
Timing: Night/upkeep  
Effect: If another player tries to poison, steal, curse, or tamper with you at night, you are alerted. Stronger versions may reveal who did it.

### Event Seal

Type: Spell  
Cost: Quartz and Ash  
Target: One event tile  
Effect: Seal an event for 1-2 days so nobody can complete it.

## Cooperation

Cooperation should be useful but not mandatory.

Cooperative event ideas:

- Multiple players contribute energy or resources.
- Rewards split by contribution, negotiation, random draw, or event text.
- Events can require roles like attacker, protector, scholar, thief, or healer.
- Players can promise help and betray during resolution.
- Some rewards are public; others are private.
- Some events are inefficient or impossible alone.

Deals are social promises, not enforced contracts.

## Betrayal And Hidden Action

Sabotage is usually anonymous. The victim normally sees the effect, not the source.

Examples:

- Poison someone's breakfast.
- Curse or steal from another player.
- Secretly alter reward splits.
- Break a cooperation promise.
- Trap or seal an event.

Counterplay:

- Alarm wards.
- Truth magic.
- Event clues.
- Defensive gear.
- Social deduction and accusations.

Most harmful effects should last one or a few days. Mild permanent curses or injuries are allowed if they create interesting choices.

## Alliances

- One player wins.
- Temporary alliances are social only.
- Deals can be broken.
- Cooperation should still be useful, even when only one player can win.

## Victory

Glory is the main score.

Players gain Glory from:

- Monster victories.
- Dangerous events.
- Quest progress.
- Public achievements.
- Impressive tricks.
- Some treasures.

Gold, Food, reagents, cards, and equipment help players earn Glory but are not usually points themselves.

Possible end conditions:

- Configured day count, default 20 days.
- Final destination after enough caravan movement.
- Shared quest completion followed by scoring.
- Boss/finale row.

## Social Play

Players are expected to use third-party voice chat. The app only needs to handle game state, actions, logs, private information, and public reveals.

Truth effects and promises are socially enforced.

## Dioxus Fullstack Notes

Server responsibilities:

- Authoritative game state.
- Lobby and room management.
- Player identity and reconnects.
- Planning submissions.
- Deterministic day resolution.
- Movement and energy cost calculation.
- Card and event effect execution.
- Event deck generation.
- Consequence card insertion.
- Public/private state projection.
- Real-time updates.

Client responsibilities:

- Lobby UI.
- Board inspection.
- Movement planning.
- Hand and inventory management.
- Trade UI.
- Ready/locked state.
- Public and private logs.
- Clear phase/timer display.

Private state must only be sent to the owning player.

## Prototype Scope

First playable prototype:

- 3-4 players.
- 20-day fixed quest.
- Configurable board, starting with 6 columns.
- One row behind the caravan, caravan row, and several future rows.
- Caravan moves 2 rows per day.
- Player board positions and energy movement.
- Energy, Gold, Food, Glory, and four reagents.
- Small deck of spells, items, wards, and equipment.
- Cooperative events.
- Sabotage cards.
- Event traps/seals.
- At least two future consequence chains.
- Public/private game log.

Prototype success criteria:

- Players understand each day.
- Concurrent turns resolve clearly.
- Movement creates real choices.
- There are more events than players can complete.
- Cooperation is useful.
- Betrayal creates suspicion without ruining turns.

## Open Questions

- Exact board depth.
- Board width scaling for 3, 4, 6, and 8 players.
- Energy recovery rate.
- Movement cost formula.
- Events completed per player per day.
- Same-event conflict resolution.
- Camp position versus current tile position.
- Consequence event timing.
- Trap visibility.
- How much planned movement is revealed before resolution.

## Next Design Tasks

- Define exact day resolution order.
- Define movement and energy costs.
- Define the first 20-30 cards.
- Define 15-20 event tiles.
- Finalize reagent names and identities.
- Define two negative and two positive consequence chains.
- Sketch main web UI screens.
- Model public and private game state.
