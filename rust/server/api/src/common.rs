use auth::Role;
use lazy_static::lazy_static;
use std::collections::hash_set::HashSet;

lazy_static! {
    pub static ref ALLOWED_EDITOR_ROLES: HashSet<Role> = [Role::Editor, Role::Admin].iter().cloned().collect();
    pub static ref ALLOWED_ADMIN_ROLES: HashSet<Role> = [Role::Admin].iter().cloned().collect();
}

pub fn clamp_optional(max: u32, value: Option<u32>) -> u32 {
    match value {
        Some(value) => if value >= max { max } else { value },
        None => max,
    }
}
