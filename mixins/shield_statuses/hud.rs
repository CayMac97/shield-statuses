// Hook helper: native_hud
fn shield_statuses_hud(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    crate::game::mod_shield_statuses_glue::on_hud(game, batch, corners);
}
