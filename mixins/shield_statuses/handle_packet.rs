fn shield_statuses_packet(
    game: &mut crate::game::Game<'_>,
    packet: &horizon_protocol::packets::play::Clientbound,
) {
    crate::game::mod_shield_statuses_glue::on_handle_packet(game, packet);
}
