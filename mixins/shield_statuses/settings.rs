// Hook helper: native_settings
fn shield_statuses_settings(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    crate::game::mod_shield_statuses_glue::on_settings(game, mod_id, buttons);
}
