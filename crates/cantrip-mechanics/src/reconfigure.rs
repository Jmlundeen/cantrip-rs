//! Reconfigure [cost] (CR 702.151a): "[Cost]: Attach this permanent to another target creature you control. Activate only
//! as a sorcery. [Cost]: Unattach this permanent. Activate only if it's attached to a creature and only as a sorcery.
//! While attached, this permanent isn't a creature." Lion Sash prints it.
//!
//! A card with reconfigure sets `reconfigure: true` (the core lets it stay attached), lists `attach(cost, text)` and
//! `unattach(cost, text)` among its `abilities` and `NOT_A_CREATURE_WHILE_ATTACHED` among its `statics`.

use crate::attach::attach_to_target;
use crate::impending::not_a_creature;
use cantrip_api::*;

fn other_creature_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id)
        if id != asker.source && q.is_on_battlefield(id) && q.controller(id) == asker.controller && q.chars(id).is(CardType::Creature))
}

/// "Attach this permanent to another target creature you control": the permanent itself is a creature until attached.
pub const fn attach(cost: ManaCost, text: &'static str) -> AbilityDef {
    ability(Cost::mana(cost), attach_to_target, text)
        .sorcery_speed()
        .targets(&[TargetSpec { min: 1, max: 1, filter: other_creature_you_control }])
}

fn unattach_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.unattach(item.object);
    Ok(())
}

fn is_attached(q: &dyn Query, asker: &Asker) -> bool {
    q.obj(asker.source).attached_to.is_some()
}

/// "Unattach this permanent": only if it is attached to a creature.
pub const fn unattach(cost: ManaCost, text: &'static str) -> AbilityDef {
    ability(Cost::mana(cost), unattach_it, text)
        .sorcery_speed()
        .condition(is_attached)
}

/// "While attached, this isn't a creature" (CR 702.151e).
fn attached(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    source == affected && q.state().obj(source).attached_to.is_some()
}

pub const NOT_A_CREATURE_WHILE_ATTACHED: StaticDef = StaticDef {
    text: "As long as this Equipment is attached to a creature, it isn't a creature.",
    layer: Layer::Type,
    affects: attached,
    apply: not_a_creature,
    ..StaticDef::BLANK
};
