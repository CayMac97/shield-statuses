# Shield Statuses for Horizon

A native Rust mixin mod for [Horizon](https://github.com/horizon-rust/horizon) (Minecraft: Java Edition 26.3) created by **CayMac**, inspired by the popular Java mod **Shield Statuses** (Walksy):

## Features

- **Live Shield Cooldown Countdown:** Shows precise countdown timer (e.g. `4.8s`, `3.2s`, `0.5s`) when your shield is disabled by an axe.
- **Real-Time Status Indicator:**
  - **Ready (Spring Green):** Shield is fully charged and ready to block.
  - **Blocking (Vivid Cyan):** Shield is raised and actively blocking damage.
  - **Disabled (Bright Red):** Shield is disabled with countdown & progress bar.
- **Durability Tracking & Low-Durability Warning:** Shows remaining durability and turns orange/red when low (< 30).
- **Multiple Placement Options:**
  - **Crosshair Anchor (Default / PvP Favorite):** Placed right beside the crosshair for zero-latency combat awareness.
  - **Offhand Slot:** Placed neatly beside the offhand hotbar slot.
  - **HUD Corners:** Top-Left, Bottom-Left, Top-Right.
- **3 Display Modes:** Full (Status + Bar + Durability), Compact (Timer + Dot), Bar Only.
- **Customizable:** Configure everything in Options > Mods > Shield Statuses.

## License

MIT © Copyright 2026 CayMac
