// Hook helper: native_key
fn shield_statuses_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    crate::game::mod_shield_statuses_glue::on_key(game, key, pressed);
}
