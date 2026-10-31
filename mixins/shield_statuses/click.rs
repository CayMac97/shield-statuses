// Hook helper: native_settings_click
fn shield_statuses_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    crate::game::mod_shield_statuses_glue::on_click(game, mod_id, button);
}
