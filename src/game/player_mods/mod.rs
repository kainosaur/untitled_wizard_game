use bevy::prelude::*;

pub mod aiming;
mod damage;
pub mod movement;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((movement::plugin, aiming::plugin, damage::plugin));
}