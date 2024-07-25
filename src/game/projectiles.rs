use std::cmp::PartialEq;

use avian2d::prelude::Collision;
use bevy::prelude::{
    in_state, App, Commands, Component, Entity, Event, EventReader, IntoSystemConfigs, Query,
    Reflect, Res, Time, Timer, TimerMode, Update,
};

use crate::game::player_mods::damage::Invincibility;
use crate::game::Damageable;
use crate::screen::GameState;
use crate::AppSet;

use super::audio::sfx::Sfx;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            tick_projectile_lifetime
                .in_set(AppSet::TickTimers)
                .run_if(in_state(GameState::Running)),
            (
                detect_projectile_collisions,
                despawn_projectiles_no_hits,
                despawn_projectiles_lifetime,
            )
                .in_set(AppSet::Update),
        ),
    );
    app.register_type::<ProjectileTeam>();
}

#[derive(Event, Debug, Clone)]
pub struct ProjectileCollisionEvent {
    #[allow(dead_code)]
    pub target: Entity,
}

#[derive(Event, Debug, Clone)]
pub struct HitByProjectileEvent {
    #[allow(dead_code)]
    pub projectile: Entity,
}

#[derive(Reflect, Clone, Debug, PartialEq, Eq)]
pub enum ProjectileTeam {
    Player,
    #[allow(dead_code)]
    Enemy,
    #[allow(dead_code)]
    Neither,
}

#[derive(Component)]
pub struct ProjectileDamage {
    pub team: ProjectileTeam,
    pub damage: f32,
    pub hits_remaining: i32, //counter for how many enemies it can hit
}

#[derive(Component)]
pub struct ProjectileLifetime {
    pub lifetime: Timer,
}

fn tick_projectile_lifetime(time: Res<Time>, mut projectile_query: Query<&mut ProjectileLifetime>) {
    for mut projectile_data in projectile_query.iter_mut() {
        projectile_data.lifetime.tick(time.delta());
    }
}

fn despawn_projectiles_lifetime(
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &ProjectileLifetime)>,
) {
    //despawn if pierce = 0 or lifetime is up
    for (entity, lifetime) in projectile_query.iter_mut() {
        if lifetime.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn despawn_projectiles_no_hits(
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &ProjectileDamage)>,
) {
    //despawn if pierce = 0 or lifetime is up
    for (entity, dmg) in projectile_query.iter_mut() {
        if dmg.hits_remaining <= 0 {
            commands.entity(entity).despawn();
        }
    }
}

// // code for only projectile hitting enemy
// fn detect_projectile_collisions (
//     mut commands: Commands,
//     mut projectile_query: Query<(Entity, &mut ProjectileDamage, &CollidingEntities)>,
//     mut enemy_query: Query<&mut Health, (With<Enemy>, Without<Player>)>,
// ) {
//     for (entity, mut projectile_damage, colliding_entities) in projectile_query.iter_mut() {
//         for &colliding_entity in colliding_entities.0.iter() {
//             if let Ok(mut health) = enemy_query.get_mut(colliding_entity) {
//                 //do damage
//                 health.0 -= projectile_damage.damage;

//                 //reduce pierce counter
//                 projectile_damage.hits_remaining -= 1;
//                 commands.trigger(Sfx::EnemyCollision);
//                 commands.trigger_targets(
//                     ProjectileCollisionEvent {
//                         target: colliding_entity,
//                     },
//                     entity,
//                 );
//             }
//         }
//     }
// }

// This breaks player/enemy collision donno why
fn detect_projectile_collisions(
    mut collision_event_reader: EventReader<Collision>,
    mut commands: Commands,
    mut projectile_query: Query<&mut ProjectileDamage>,
    mut health_havers: Query<(&mut Damageable, Option<&Invincibility>)>,
) {
    //datastructure to keep track of hit entities, as they cant be hit more than once per frame
    let mut hit_entities = Vec::new();

    for Collision(contacts) in collision_event_reader.read() {
        let (Some(col1), Some(col2)) = (contacts.body_entity1, contacts.body_entity2) else {
            continue;
        };

        //check if one is the projectile and the other the health_haver
        let (projectile_entity, health_entity) =
            if projectile_query.get(col1).is_ok() && health_havers.get(col2).is_ok() {
                (col1, col2)
            } else if projectile_query.get(col2).is_ok() && health_havers.get(col1).is_ok() {
                (col2, col1)
            } else {
                continue;
            };

        let Ok((mut health, invinicibility)) = health_havers.get_mut(health_entity) else {
            return;
        };

        let Ok(mut projectile_dmg) = projectile_query.get_mut(projectile_entity) else {
            return;
        };

        //can this projectile damage this entity?
        if health.team == projectile_dmg.team {
            continue;
        }

        //is this entity invulnerable
        if invinicibility.is_some() {
            continue;
        }

        //has this entity already been hit this frame?
        if hit_entities.contains(&health_entity) {
            continue;
        }

        hit_entities.push(health_entity);

        //do damage + health.invincibility_timer)
        health.health -= projectile_dmg.damage;
        commands.entity(health_entity).insert(Invincibility {
            timer: Timer::new(health.invincibility_timer, TimerMode::Once),
        });

        //reduce projectile pierce counter
        projectile_dmg.hits_remaining -= 1;

        commands.trigger(Sfx::EnemyCollision);
        commands.trigger_targets(
            ProjectileCollisionEvent {
                target: health_entity,
            },
            projectile_entity,
        );
        commands.trigger_targets(
            HitByProjectileEvent {
                projectile: projectile_entity,
            },
            health_entity,
        );
    }
}
