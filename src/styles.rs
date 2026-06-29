pub const BASE_CSS: &str = r#"
body { margin: 0; background: #15140f; color: #f3ead7; font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }
button, input, select, textarea { font: inherit; }
h1, h2, p { margin: 0; }
h1 { font-size: 24px; }
h2 { font-size: 16px; margin-bottom: 10px; }
input, select, textarea { border: 1px solid #5b513d; border-radius: 6px; background: #100f0c; color: #f3ead7; padding: 8px 10px; }
textarea { min-height: 70px; resize: vertical; }
button { border: 1px solid #6f6046; border-radius: 6px; background: #30291d; color: #f3ead7; padding: 8px 10px; cursor: pointer; }
button:hover { background: #403723; }
.primary { background: #7c5f2d; border-color: #a88748; }
.selected { outline: 2px solid #d2a94f; }
.private-log { color: #d5b7ff !important; }
@media (max-width: 980px) { .responsive-layout { grid-template-columns: 1fr !important; } .side-panel { grid-row: 1; } }
"#;

pub const SHELL: &str = "min-height: 100vh;";
pub const TOPBAR: &str = "display: flex; justify-content: space-between; gap: 16px; padding: 18px 22px; border-bottom: 1px solid #393225; background: #211d16;";
pub const JOIN: &str = "display: flex; flex-wrap: wrap; align-items: center; gap: 8px;";
pub const MUTED: &str = "color: #b9aa91; margin-bottom: 12px;";
pub const LAYOUT: &str =
    "display: grid; grid-template-columns: minmax(0, 1fr) 360px; gap: 18px; padding: 18px 22px;";
pub const BOARD_PANEL: &str = "overflow-x: auto;";
pub const BOARD: &str = "min-width: 760px; display: grid; gap: 8px;";
pub const BOARD_ROW: &str = "display: grid; grid-template-columns: 90px repeat(7, minmax(92px, 1fr)); gap: 8px; align-items: stretch;";
pub const ROW_LABEL: &str = "color: #c8b891; padding-top: 10px;";
pub const ROW_LABEL_SUB: &str = "display: block; color: #82745d; font-size: 12px;";
pub const TILE: &str = "min-height: 98px; text-align: left; display: flex; flex-direction: column; gap: 5px; background: #242018;";
pub const COORDS: &str = "color: #9d8f77; font-size: 12px;";
pub const SEALED: &str = "color: #87c7ff; font-size: 12px;";
pub const OCCUPANTS: &str = "margin-top: auto; display: flex; gap: 4px;";
pub const OCCUPANT: &str = "display: inline-grid; place-items: center; width: 22px; height: 22px; border-radius: 50%; background: #8d3f43; font-size: 12px;";
pub const SIDE: &str = "display: grid; gap: 12px; align-content: start;";
pub const PANEL: &str =
    "border: 1px solid #393225; border-radius: 8px; background: #201c15; padding: 14px;";
pub const LABEL: &str = "display: block; color: #c8b891; font-size: 13px; margin: 10px 0 5px;";
pub const FULL_CONTROL: &str = "width: 100%; box-sizing: border-box;";
pub const PLAYER_LINE: &str = "display: grid; grid-template-columns: 1fr auto auto auto auto auto; gap: 8px; align-items: center; border-top: 1px solid #393225; padding: 8px 0;";
pub const SMALL_TEXT: &str = "color: #c8b891; font-size: 13px;";
pub const MOVE_SUMMARY: &str = "display: grid; gap: 4px; padding: 10px; border: 1px solid #4a402f; border-radius: 6px; background: #18150f; margin-bottom: 10px;";
pub const SELECTED_CARD: &str = "display: grid; gap: 6px; padding: 10px; border: 1px solid #695739; border-radius: 6px; background: #2a2318; margin: 10px 0;";
pub const TARGET_LIST: &str = "display: flex; flex-wrap: wrap; gap: 8px; margin-bottom: 8px;";
pub const HAND_GRID: &str = "display: grid; grid-template-columns: repeat(auto-fill, minmax(150px, 1fr)); gap: 8px; margin-top: 10px;";
pub const HAND_BAR: &str =
    "border-top: 1px solid #393225; background: #18150f; padding: 14px 22px 18px;";
pub const HAND_CARD: &str = "min-height: 138px; display: grid; align-content: start; gap: 6px; text-align: left; background: #262116;";
pub const CARD_KIND: &str = "color: #9d8f77; font-size: 11px; text-transform: uppercase;";
pub const CARD_COST: &str = "align-self: end; color: #f0cf80; font-size: 12px;";
pub const MOVE_COST: &str = "color: #9d8f77; font-size: 12px;";
pub const MOVE_COST_OK: &str = "color: #9bdc8a; font-size: 12px; font-weight: 700;";
pub const LOG_PANEL: &str = "border: 1px solid #393225; border-radius: 8px; background: #201c15; padding: 14px; margin: 0 22px 22px;";
pub const LOG_ENTRY: &str = "color: #d6c8ad; border-top: 1px solid #332d22; padding: 7px 0;";
pub const EMPTY: &str = "padding: 40px 22px; color: #c8b891;";
pub const MODAL: &str = "position: fixed; inset: 0; display: grid; place-items: center; background: rgba(0, 0, 0, 0.5);";
pub const MODAL_BOX: &str =
    "border: 1px solid #a88748; background: #211d16; border-radius: 8px; padding: 24px;";
