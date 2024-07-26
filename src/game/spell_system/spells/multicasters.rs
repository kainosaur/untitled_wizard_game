use std::slice::Iter;
use std::sync::Arc;

use bevy::log::info;
use bevy::math::Vec2;
use bevy::prelude::{Entity, World};
use log::warn;

use crate::game::spell_system::{SpellComponent, SpellData, SpellEffect, SpellModifier};
use crate::game::spell_system::casting::{
    SpellCastContext, SpellCaster,
};
use crate::game::spell_system::triggers::{CollisionSpellTrigger, do_collision_trigger};

pub(super) fn get_spells() -> Vec<SpellComponent> {
    vec![SpellComponent {
        data: Box::new(TriggerSpellData {
            spells_triggered: 1,
        }),
        icon_id: 24,
    }]
}

#[derive(Clone)]
pub struct TriggerSpellData {
    pub spells_triggered: usize,
}
impl SpellData for TriggerSpellData {
    fn build(&self, iter: &mut Iter<SpellComponent>) -> Option<Arc<dyn SpellEffect>> {
        let trigger_spell = iter.next()?.data.build(iter)?;
        let mut spells_triggered: Vec<Arc<dyn SpellEffect>> = Vec::new();


        for _ in 0..self.spells_triggered {
            let Some(next) = iter.next() else {
                warn!("Failed to build trigger's child spell, not enough spells in the list.");
                break;
            }; //no more spell_system left to add to this trigger

            let Some(spell) = next.data.build(iter) else {
                warn!("failed to build trigger's child spell, failed to build child spell");
                break;
            }; //failed to build child spell

            spells_triggered.push(spell);
        }

        Some(Arc::new(TriggerSpell {
            trigger_spell,
            spells_triggered: Arc::new(spells_triggered),
        }))
    }

    fn get_name(&self) -> String {
        "Collision Trigger".to_string()
    }

    fn get_desc(&self) -> String {
        "When the following spell's projectiles collide with something, they cast the immediately following spell.".to_string()
    }
}
#[derive(Debug, Clone)]
pub struct TriggerSpell {
    pub trigger_spell: Arc<dyn SpellEffect>,
    pub spells_triggered: Arc<Vec<Arc<dyn SpellEffect>>>,
}
impl SpellEffect for TriggerSpell {
    fn cast(&self, context: &mut SpellCastContext, world: &mut World) {
        let spells = self.spells_triggered.clone();
        let new_context = context.fresh_clone();
        let modifier: SpellModifier = Box::new(move |e: Entity, mod_world: &mut World| {
            let mut spell_context = new_context.clone();
            spell_context.caster = e;
            mod_world.entity_mut(e).insert((
                CollisionSpellTrigger {
                    values: spell_context.values.clone(),
                    spells: spells.clone(),
                },
            ));
            mod_world.entity_mut(e).observe(do_collision_trigger);
        });
        context.add_modifier("CollisionTrigger", modifier);
        self.trigger_spell.cast(context, world);
    }
}
