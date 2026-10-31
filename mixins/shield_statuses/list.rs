// Hook helper: native_mods
fn shield_statuses_list(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    crate::game::mod_shield_statuses_glue::on_list(mods);
}
