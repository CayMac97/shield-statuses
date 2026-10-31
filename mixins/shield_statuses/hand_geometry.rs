fn shield_statuses_hand(game: &mut crate::game::Game<'_>) {
    crate::game::mod_shield_statuses_glue::on_hand_geometry(game);
}
