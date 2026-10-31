fn shield_statuses_armor_stand(a: &horizon_client::entities::ArmorStand) {
    crate::game::mod_shield_statuses_glue::on_armor_stand_items(a);
}
