// Shield Statuses runtime module
// Shimmers 3D shield model: Green when ready to block, Red when broken/on cooldown.
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::Instant;

pub const ID: &str = "shield_statuses";
pub const NAME: &str = "Shield Statuses";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str =
    "3D Shield indicator: shimmers green when ready to block and red when broken or on cooldown.";

pub const KEY_TOGGLE: &str = "toggle";
pub const KEY_TOGGLE_LABEL: &str = "Toggle Shield Statuses";
pub const KEY_TOGGLE_DEFAULT: &str = "key.keyboard.unknown";

static ENABLED: AtomicBool = AtomicBool::new(true);
static PULSE_ANIMATION: AtomicBool = AtomicBool::new(true);
static ENEMY_SHIELDS: AtomicBool = AtomicBool::new(true);

// 0 represents local player first person; any other value represents a specific entity ID.
static CURRENT_ENTITY: AtomicU32 = AtomicU32::new(0);

// Tracks entity IDs whose shields are disabled and their expiry time.
static DISABLED_ENTITIES: Mutex<Option<HashMap<u32, Instant>>> = Mutex::new(None);

pub fn set_current_entity(id: u32) {
    CURRENT_ENTITY.store(id, Ordering::Relaxed);
}

pub fn current_entity() -> u32 {
    CURRENT_ENTITY.load(Ordering::Relaxed)
}

pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

pub fn toggle_enabled() -> bool {
    let prev = ENABLED.fetch_xor(true, Ordering::Relaxed);
    !prev
}

pub fn pulse_enabled() -> bool {
    PULSE_ANIMATION.load(Ordering::Relaxed)
}

pub fn toggle_pulse() -> bool {
    let prev = PULSE_ANIMATION.fetch_xor(true, Ordering::Relaxed);
    !prev
}

pub fn enemy_shields_enabled() -> bool {
    ENEMY_SHIELDS.load(Ordering::Relaxed)
}

pub fn toggle_enemy_shields() -> bool {
    let prev = ENEMY_SHIELDS.fetch_xor(true, Ordering::Relaxed);
    !prev
}

pub fn disable_entity(id: u32, duration_secs: f32) {
    let expiry = Instant::now() + std::time::Duration::from_secs_f32(duration_secs);
    if let Ok(mut lock) = DISABLED_ENTITIES.lock() {
        let map = lock.get_or_insert_with(HashMap::new);
        map.insert(id, expiry);
    }
}

pub fn is_entity_disabled(id: u32) -> bool {
    if let Ok(mut lock) = DISABLED_ENTITIES.lock() {
        if let Some(map) = lock.as_mut() {
            let now = Instant::now();
            if let Some(expiry) = map.get(&id) {
                if now < *expiry {
                    return true;
                } else {
                    map.remove(&id);
                }
            }
        }
    }
    false
}
