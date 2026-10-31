fn shield_statuses_submit_item_lit(
    game: &crate::game::Game<'_>,
    state: &horizon_client::item_render::ItemRenderState,
    pose: glam::Mat4,
    light: u32,
    lights: (glam::Vec3, glam::Vec3),
    out: &mut Vec<horizon_client::mesher::Vertex>,
) -> std::ops::ControlFlow<()> {
    crate::game::mod_shield_statuses_glue::on_submit_item_lit(
        game, state, pose, light, lights, out,
    )
}
