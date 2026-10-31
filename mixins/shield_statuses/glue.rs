// Glue between Horizon's `Game` and `mod_shield_statuses`
use crate::game::Game;
use crate::game::mod_shield_statuses as m;
use crate::game::native_hooks::NativeMod;
use std::ops::ControlFlow;

pub(super) fn on_submit_item_lit(
    game: &Game<'_>,
    state: &horizon_client::item_render::ItemRenderState,
    pose: glam::Mat4,
    light: u32,
    lights: (glam::Vec3, glam::Vec3),
    out: &mut Vec<horizon_client::mesher::Vertex>,
) -> ControlFlow<()> {
    if !game.native_on(m::ID) || !m::is_enabled() {
        return ControlFlow::Continue(());
    }

    let is_shield = state.layers.iter().any(|layer| match &layer.geometry {
        horizon_client::item_render::LayerGeometry::Special { model, .. } => {
            matches!(model.kind, horizon_assets::item_model::SpecialKind::Shield)
        }
        _ => false,
    });

    if !is_shield {
        return ControlFlow::Continue(());
    }

    let start_idx = out.len();

    let mut sink = horizon_client::item_render::WorldSink {
        out,
        light,
        lights: Some(lights),
        glint: game.glint(),
    };
    state.submit(
        &game.pack.item_renderer,
        &game.gui.atlas.sprites,
        &game.item_host(),
        pose,
        &mut sink,
    );

    apply_shield_shimmer(game, start_idx, out);
    ControlFlow::Break(())
}

pub(super) fn on_submit_item_unlit(
    game: &Game<'_>,
    state: &horizon_client::item_render::ItemRenderState,
    pose: glam::Mat4,
    light: u32,
    out: &mut Vec<horizon_client::mesher::Vertex>,
) -> ControlFlow<()> {
    if !game.native_on(m::ID) || !m::is_enabled() {
        return ControlFlow::Continue(());
    }

    let is_shield = state.layers.iter().any(|layer| match &layer.geometry {
        horizon_client::item_render::LayerGeometry::Special { model, .. } => {
            matches!(model.kind, horizon_assets::item_model::SpecialKind::Shield)
        }
        _ => false,
    });

    if !is_shield {
        return ControlFlow::Continue(());
    }

    let start_idx = out.len();

    let mut sink = horizon_client::item_render::WorldSink {
        out,
        light,
        lights: None,
        glint: game.glint(),
    };
    state.submit(
        &game.pack.item_renderer,
        &game.gui.atlas.sprites,
        &game.item_host(),
        pose,
        &mut sink,
    );

    apply_shield_shimmer(game, start_idx, out);
    ControlFlow::Break(())
}

fn apply_shield_shimmer(
    game: &Game<'_>,
    start_idx: usize,
    out: &mut [horizon_client::mesher::Vertex],
) {
    // If the shield is actively raised and blocking attacks, it is functional!
    if m::current_blocking() {
        tint_shield(game, start_idx, out, false);
        return;
    }

    let entity_id = m::current_entity();
    let local_id = game
        .player_model
        .as_ref()
        .map(|p| p.id)
        .or_else(|| game.remote.as_ref().map(|r| r.entity_id as u32))
        .unwrap_or(0);

    let is_local = entity_id == 0 || entity_id == local_id;

    let is_disabled = if is_local {
        // If local player is using/blocking, definitely ready!
        if game.using.is_some() {
            false
        } else {
            // Check all sources of cooldown for local player:
            let offhand_cd = game.offhand.as_ref().is_some_and(|s| {
                game.cooldowns.is_on_cooldown(&s.item)
                    || game.cooldowns.is_on_cooldown(s.item.trim_start_matches("minecraft:"))
            });
            let mainhand_cd = game
                .inventory
                .get(game.selected)
                .and_then(Option::as_ref)
                .is_some_and(|s| {
                    game.cooldowns.is_on_cooldown(&s.item)
                        || game.cooldowns.is_on_cooldown(s.item.trim_start_matches("minecraft:"))
                });

            game.shield_cooldown > 0
                || game.cooldowns.is_on_cooldown("minecraft:shield")
                || game.cooldowns.is_on_cooldown("shield")
                || offhand_cd
                || mainhand_cd
                || m::is_local_disabled()
                || (local_id != 0 && m::is_entity_disabled(local_id))
        }
    } else {
        if !m::enemy_shields_enabled() {
            return;
        }
        m::is_entity_disabled(entity_id)
    };

    tint_shield(game, start_idx, out, is_disabled);
}

fn tint_shield(
    game: &Game<'_>,
    start_idx: usize,
    out: &mut [horizon_client::mesher::Vertex],
    is_disabled: bool,
) {
    let time = game.started.elapsed().as_secs_f32();
    let pulse = if m::pulse_enabled() {
        1.0 + 0.12 * (time * 4.5).sin()
    } else {
        1.0
    };

    for v in &mut out[start_idx..] {
        let [r, g, b, a] = v.color.to_le_bytes();
        let (nr, ng, nb) = if is_disabled {
            // Vivid Crimson Red glow (matching user sample)
            let nr = (((r as f32) * 1.30 + 35.0) * pulse).clamp(0.0, 255.0) as u8;
            let ng = ((g as f32) * 0.05).clamp(0.0, 255.0) as u8;
            let nb = (((b as f32) * 0.20 + 8.0) * pulse).clamp(0.0, 255.0) as u8;
            (nr, ng, nb)
        } else {
            // Vivid Emerald Green glow (matching user sample)
            let nr = (((r as f32) * 0.18 + 5.0) * pulse).clamp(0.0, 255.0) as u8;
            let ng = (((g as f32) * 1.30 + 35.0) * pulse).clamp(0.0, 255.0) as u8;
            let nb = (((b as f32) * 0.22 + 8.0) * pulse).clamp(0.0, 255.0) as u8;
            (nr, ng, nb)
        };
        v.color = u32::from_le_bytes([nr, ng, nb, a]);
    }
}

pub(super) fn on_hand_geometry(game: &mut Game<'_>) {
    m::set_current_entity(0);
    m::set_current_blocking(game.using.is_some());
}

pub(super) fn on_held_item_geometry(mob: &horizon_client::mobs::Mob) {
    m::set_current_entity(mob.id);
    let is_blocking = mob.extra.blocking || (mob.extra.using_item > 0 && mob.extra.using_off_hand);
    if is_blocking {
        // If the entity is actively raising/using their shield, clear disabled state!
        m::clear_entity_disabled(mob.id);
    }
    m::set_current_blocking(is_blocking);
}

pub(super) fn on_armor_stand_items(a: &horizon_client::entities::ArmorStand) {
    m::set_current_entity(a.id);
    m::set_current_blocking(false);
}

pub(super) fn on_handle_packet(
    game: &mut Game<'_>,
    packet: &horizon_protocol::packets::play::Clientbound,
) {
    use horizon_protocol::packets::play::Clientbound as C;
    let local_id = game
        .player_model
        .as_ref()
        .map(|p| p.id)
        .or_else(|| game.remote.as_ref().map(|r| r.entity_id as u32))
        .unwrap_or(0);

    match packet {
        C::EntityEvent(p) => {
            let id = p.id as u32;
            if p.event == 30 {
                // Status 30: Shield broken / disabled by an axe hit
                m::disable_entity(id, 5.0);
                if id == local_id {
                    m::set_local_disabled(true, 5.0);
                }
            } else if p.event == 29 {
                // Status 29: Shield blocked an attack successfully -> definitely functional!
                m::clear_entity_disabled(id);
                if id == local_id {
                    m::set_local_disabled(false, 0.0);
                }
            }
        }
        C::Cooldown(p) => {
            // When the server sets an item cooldown for shields on the local player:
            let g = p.group.as_str();
            if g == "minecraft:shield" || g == "shield" || g.ends_with("shield") {
                if p.duration > 0 {
                    let secs = p.duration as f32 / 20.0;
                    m::set_local_disabled(true, secs);
                    if local_id != 0 {
                        m::disable_entity(local_id, secs);
                    }
                } else {
                    m::set_local_disabled(false, 0.0);
                    if local_id != 0 {
                        m::clear_entity_disabled(local_id);
                    }
                }
            }
        }
        _ => {}
    }
}

pub(super) fn on_key(game: &mut Game<'_>, key: &str, pressed: bool) {
    if !pressed || !game.native_on(m::ID) {
        return;
    }
    let toggle_key = game.native_mod_key(m::ID, m::KEY_TOGGLE, m::KEY_TOGGLE_DEFAULT);
    if key == toggle_key && toggle_key != "key.keyboard.unknown" {
        m::toggle_enabled();
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

pub(super) fn on_settings(_game: &mut Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    if mod_id != m::ID {
        return;
    }
    let enabled_str = if m::is_enabled() { "ON" } else { "OFF" };
    let pulse_str = if m::pulse_enabled() { "PULSE" } else { "STATIC" };
    let enemy_str = if m::enemy_shields_enabled() { "ON" } else { "OFF" };

    buttons.push(format!("Mod Status: {enabled_str}"));
    buttons.push(format!("Shimmer Effect: {pulse_str}"));
    buttons.push(format!("Enemy Shields: {enemy_str}"));
}

pub(super) fn on_click(
    _game: &mut Game<'_>,
    mod_id: &str,
    button: usize,
) {
    if mod_id != m::ID {
        return;
    }
    match button {
        0 => {
            m::toggle_enabled();
        }
        1 => {
            m::toggle_pulse();
        }
        2 => {
            m::toggle_enemy_shields();
        }
        _ => {}
    }
}
