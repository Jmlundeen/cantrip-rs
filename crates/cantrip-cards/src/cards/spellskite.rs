use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Change a target of target spell or ability to this creature" (CR 115.7b): the core changes one target, if this creature
/// is still on the battlefield and a legal target for it (CR 115.7a); with several that could change, this ability's
/// controller chooses which. Activating it needs neither (the Spellskite rulings): a spell or ability with no target, or none
/// this creature could be, is still a legal target, and nothing changes.
fn redirect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.change_a_target_to(item.targets[0], item.object, item.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{U/P}"), redirect, "{U/P}: Change a target of target spell or ability to this creature.").targets(&[targets::SPELL_OR_ABILITY])],
    ..oracle::SPELLSKITE
};
