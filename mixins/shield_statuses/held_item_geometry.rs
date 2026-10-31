fn shield_statuses_held(mob: &horizon_client::mobs::Mob) {
    crate::game::mod_shield_statuses_glue::on_held_item_geometry(mob);
}
