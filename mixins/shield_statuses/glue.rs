// Glue between Horizon's `Game` and `mod_shield_statuses`
use crate::game::Game;
use crate::game::mod_shield_statuses as m;
use crate::game::native_hooks::NativeMod;
use horizon_client::extras_hud::Corners;
use horizon_client::hud::HudBatch;

pub(super) fn on_hud(
    game: &mut Game<'_>,
    batch: &mut HudBatch,
    _corners: &mut Corners,
) {
    if !game.native_on(m::ID) || game.player.is_none() {
        return;
    }
    let s = {
        let g = &*game;
        m::settings(|k| g.native_setting(m::ID, k))
    };
    if !s.enabled {
        return;
    }

    // Only render in first person, not spectator, not in debug F3
    if !game.camera_type.first_person() || game.spectator() || game.debug_crosshair() {
        return;
    }

    // Check if player has a shield equipped (offhand or main hand)
    let offhand_shield = game.offhand.as_ref().filter(|st| st.item.ends_with("shield") || game.blocks_attacks(st).is_some());
    let mainhand_shield = game.inventory.get(game.selected).and_then(Option::as_ref).filter(|st| st.item.ends_with("shield") || game.blocks_attacks(st).is_some());

    let shield_stack = offhand_shield.or(mainhand_shield);

    let target_in_reach = game.crosshair_living().is_some();
    if shield_stack.is_none() && (!s.show_target || !target_in_reach) {
        return;
    }

    let (wi, hi) = game.gui_size();
    if wi <= 0.0 || hi <= 0.0 {
        return;
    }

    // Determine state of own shield
    let (status_text, status_color, cooldown_prog, is_on_cooldown) = if let Some(_stack) = shield_stack {
        let cd_ticks = game.shield_cooldown;
        if cd_ticks > 0 {
            let secs = cd_ticks as f32 / 20.0;
            let prog = 1.0 - (cd_ticks as f32 / 100.0).clamp(0.0, 1.0);
            (format!("{:.1}s", secs), m::COLOR_DISABLED, prog, true)
        } else if game.using.is_some() {
            ("BLOCKING".to_string(), m::COLOR_BLOCKING, 1.0, false)
        } else {
            ("READY".to_string(), m::COLOR_READY, 1.0, false)
        }
    } else {
        ("NONE".to_string(), m::COLOR_WHITE, 0.0, false)
    };

    // Durability of own shield
    let dur_text = if s.show_durability {
        if let Some(stack) = shield_stack {
            let max_dur = game.items().get(&stack.item).map_or(336, |i| i.max_damage);
            let cur_dur = max_dur.saturating_sub(stack.damage);
            Some(cur_dur)
        } else {
            None
        }
    } else {
        None
    };

    // Calculate center coordinates
    let cx = ((wi - 15.0) / 2.0).floor() + 7.0;
    let cy = ((hi - 15.0) / 2.0).floor() + 7.0;

    let (base_x, base_y) = match s.position {
        m::Position::Crosshair => {
            // Center of screen, 16px to the right of crosshair
            (cx + 16.0, cy - 8.0)
        }
        m::Position::Offhand => {
            let center = (wi * 0.5).floor();
            let is_left = !game.settings.main_hand_left;
            let offhand_x = if is_left { center - 120.0 } else { center + 104.0 };
            (offhand_x, hi - 24.0)
        }
        m::Position::TopLeft => (8.0, 8.0),
        m::Position::BottomLeft => (8.0, hi - 36.0),
        m::Position::TopRight => (wi - 68.0, 8.0),
    };

    // 1. Render player's own shield badge
    if shield_stack.is_some() {
        match s.mode {
            m::DisplayMode::Full => {
                let badge_w = 52.0;
                let badge_h = 16.0;

                // Dark background
                game.native_fill(batch, [base_x, base_y, badge_w, badge_h], m::COLOR_BG);
                // Status color accent bar on left edge
                game.native_fill(batch, [base_x, base_y, 2.0, badge_h], status_color);

                // Status text (READY / BLOCKING / 4.2s)
                game.native_text(batch, &status_text, [base_x + 5.0, base_y + 2.0], status_color, true);

                // Durability count on the right
                if let Some(dur) = dur_text {
                    let dur_str = dur.to_string();
                    let dur_w = game.native_text_width(&dur_str);
                    let dur_color = if dur < 30 { m::COLOR_WARNING } else { 0xFFAAAAAA };
                    game.native_text(batch, &dur_str, [base_x + badge_w - dur_w - 3.0, base_y + 2.0], dur_color, true);
                }

                // Cooldown recovery progress bar (if on cooldown)
                if is_on_cooldown {
                    let bar_w = badge_w - 6.0;
                    game.native_fill(batch, [base_x + 4.0, base_y + 12.0, bar_w, 2.0], m::COLOR_BAR_BG);
                    let fill_w = (bar_w * cooldown_prog).clamp(0.0, bar_w);
                    if fill_w > 0.0 {
                        game.native_fill(batch, [base_x + 4.0, base_y + 12.0, fill_w, 2.0], status_color);
                    }
                }
            }
            m::DisplayMode::Compact => {
                if is_on_cooldown {
                    game.native_fill(batch, [base_x, base_y, 28.0, 12.0], m::COLOR_BG);
                    game.native_text(batch, &status_text, [base_x + 2.0, base_y + 2.0], status_color, true);
                } else {
                    // Small 5x5 indicator pip
                    game.native_fill(batch, [base_x, base_y + 5.0, 5.0, 5.0], status_color);
                }
            }
            m::DisplayMode::BarOnly => {
                if is_on_cooldown {
                    let bar_w = 20.0;
                    let bar_x = (cx - bar_w / 2.0).floor();
                    let bar_y = cy + 14.0;
                    game.native_fill(batch, [bar_x, bar_y, bar_w, 2.0], m::COLOR_BAR_BG);
                    let fill_w = (bar_w * cooldown_prog).clamp(0.0, bar_w);
                    if fill_w > 0.0 {
                        game.native_fill(batch, [bar_x, bar_y, fill_w, 2.0], status_color);
                    }
                }
            }
        }
    }

    // 2. Render target shield indicator when hovering living target in reach
    if s.show_target && target_in_reach {
        // Target indicator placed above crosshair
        let tag = "SHIELD READY";
        let tag_w = game.native_text_width(tag);
        let tag_x = (cx - tag_w / 2.0).floor();
        let tag_y = cy - 16.0;
        game.native_fill(batch, [tag_x - 3.0, tag_y - 1.0, tag_w + 6.0, 11.0], m::COLOR_BG);
        game.native_text(batch, tag, [tag_x, tag_y], m::COLOR_READY, true);
    }
}

pub(super) fn on_key(game: &mut Game<'_>, key: &str, pressed: bool) {
    if !pressed || !game.native_on(m::ID) || !game.native_playing() {
        return;
    }
    let mut s = {
        let g = &*game;
        m::settings(|k| g.native_setting(m::ID, k))
    };

    let toggle_key = game.native_mod_key(m::ID, m::KEY_TOGGLE, m::KEY_TOGGLE_DEFAULT);
    if key == toggle_key && toggle_key != "key.keyboard.unknown" {
        s.enabled = !s.enabled;
        m::set_settings(s.clone());
        for (k, v) in s.stored() {
            game.native_set_setting(m::ID, k, &v);
        }
    }
}

pub(super) fn on_list(mods: &mut Vec<NativeMod>) {
    mods.push(NativeMod {
        id: m::ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: vec![[
            m::KEY_TOGGLE.into(),
            m::KEY_TOGGLE_LABEL.into(),
            m::KEY_TOGGLE_DEFAULT.into(),
        ]],
    });
}

pub(super) fn on_settings(game: &mut Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    if mod_id != m::ID {
        return;
    }
    let g = &*game;
    let s = m::settings(|key| g.native_setting(m::ID, key));
    buttons.extend(s.labels());
}

pub(super) fn on_click(game: &mut Game<'_>, mod_id: &str, button: usize) {
    if mod_id != m::ID {
        return;
    }
    let mut s = {
        let g = &*game;
        m::settings(|key| g.native_setting(m::ID, key))
    };
    s.click(button);
    m::set_settings(s.clone());
    for (key, value) in s.stored() {
        game.native_set_setting(m::ID, key, &value);
    }
}
