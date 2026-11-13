use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip, other_creatures_with_equipment_of_subtype};
use cantrip_mechanics::statics::grant_delayed_ability;
use cantrip_mechanics::triggers::{CAST_THIS_SPELL, THIS_ATTACKS};
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

const GRANTED: &str = "Whenever this creature attacks while you control another creature with a Communicator attached to it, this creature can't be blocked this turn.";

/// Copy this spell (CR 707.10): the copy of a permanent spell becomes a token as it resolves (CR 707.10f).
fn copy_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.copy_spell(item.object, item.controller, false)?;
    Ok(())
}

/// "while you control another creature with a Communicator attached to it" (CR 603.2): part of the trigger condition, asked
/// with the attack only and never again as the ability resolves (no intervening "if", CR 603.4), about a creature other
/// than the attacking one that has an Equipment with the Communicator subtype attached to it (CR 301.5, 205.3g).
fn another_creature_has_a_communicator(q: &dyn Query, asker: &Asker) -> bool {
    other_creatures_with_equipment_of_subtype(q, asker.controller, asker.source, Subtype::Communicator) > 0
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(CAST_THIS_SPELL, copy_it, "When you cast this spell, copy it. (The copy becomes a token.)"),
        equip(mana("{2}"), "Equip {2}"),
    ],
    // The ability the equipped creature has (CR 613.1f): the creature's own, so its controller controls it (CR 113.8).
    delayed: &[triggered(THIS_ATTACKS, effects::this_cant_be_blocked_this_turn, GRANTED).only_if(another_creature_has_a_communicator)],
    statics: &[static_ability(
        Layer::Ability,
        attached_object,
        grant_delayed_ability,
        "Equipped creature has \"Whenever this creature attacks while you control another creature with a Communicator attached to it, this creature can't be blocked this turn.\"",
    )],
    ..oracle::COMBADGE
};
