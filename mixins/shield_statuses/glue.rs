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
    let entity_id = m::current_entity();
    let local_id = game
        .player_model
        .as_ref()
        .map(|p| p.id)
        .or_else(|| game.remote.as_ref().map(|r| r.entity_id as u32))
        .unwrap_or(0);

    let is_local = entity_id == 0 || entity_id == local_id;

    let is_disabled = if is_local {
        game.shield_cooldown > 0
    } else {
        if !m::enemy_shields_enabled() {
            return;
        }
        m::is_entity_disabled(entity_id)
    };

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

pub(super) fn on_hand_geometry(_game: &mut Game<'_>) {
    m::set_current_entity(0);
}

pub(super) fn on_held_item_geometry(mob: &horizon_client::mobs::Mob) {
    m::set_current_entity(mob.id);
}

pub(super) fn on_armor_stand_items(a: &horizon_client::entities::ArmorStand) {
    m::set_current_entity(a.id);
}

pub(super) fn on_handle_packet(
    game: &mut Game<'_>,
    packet: &horizon_protocol::packets::play::Clientbound,
) {
    use horizon_protocol::packets::play::Clientbound as C;
    match packet {
        C::EntityEvent(p) => {
            // Status 30 in Minecraft: LivingEntity.handleEntityEvent(30) -> Shield break / disabled
            if p.event == 30 {
                m::disable_entity(p.id as u32, 5.0);
            }
        }
        C::Sound(p) => {
            if let Some(name) = game.remote_sound_name(&p.sound) {
                if name.ends_with("shield.break") || name.contains("shield.break") {
                    let at = [
                        f64::from(p.x) / 8.0,
                        f64::from(p.y) / 8.0,
                        f64::from(p.z) / 8.0,
                    ];
                    let mut closest: Option<(u32, f64)> = None;
                    for mob in &game.mobs {
                        let dx = mob.position.x - at[0];
                        let dy = mob.position.y - at[1];
                        let dz = mob.position.z - at[2];
                        let dist_sq = dx * dx + dy * dy + dz * dz;
                        if dist_sq < 64.0 && closest.map_or(true, |(_, d)| dist_sq < d) {
                            closest = Some((mob.id, dist_sq));
                        }
                    }
                    if let Some((id, _)) = closest {
                        m::disable_entity(id, 5.0);
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
