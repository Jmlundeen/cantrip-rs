use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip, other_creatures_with_equipment_of_subtype};
use cantrip_mechanics::effects;
use cantrip_mechanics::statics::grant_delayed_ability;
use cantrip_mechanics::triggers::{CAST_THIS_SPELL, THIS_ATTACKS};
use cantrip_oracle as oracle;

const GRANTED: &str = "Whenever this creature attacks, draw X cards, then you may put up to X land cards from your hand onto the battlefield tapped, where X is the number of other creatures you control with a Communicator attached to them.";

/// Copy this spell (CR 707.10): the copy of a permanent spell becomes a token as it resolves (CR 707.10f).
fn copy_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.copy_spell(item.object, item.controller, false)?;
    Ok(())
}

/// "Whenever this creature attacks, draw X cards, then you may put up to X land cards from your hand onto the battlefield
/// tapped" (CR 121.1, 608.2c, 608.2d): X is fixed once, as the trigger resolves, and both halves use it; the draw ends the
/// sentence, so the cards just drawn are in the hand to choose from (CR 608.2c), and the "up to" is answered with any number
/// from none to X.
fn attack_draw_then_may_put_lands_tapped(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    // X: "the number of other creatures you control with a Communicator attached to them", counted as it resolves (CR 608.2h).
    let x = other_creatures_with_equipment_of_subtype(&*g, item.controller, item.object, Subtype::Communicator);
    effects::draw_then_may_put_up_to_lands_tapped(g, item.controller, item.object, x)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(CAST_THIS_SPELL, copy_it, "When you cast this spell, copy it. (The copy becomes a token.)"),
        equip(mana("{1}"), "Equip {1}"),
    ],
    // The ability the equipped creature has (CR 613.1f): the creature's own, so whoever controls the creature controls the
    // trigger and counts their own creatures (CR 113.8).
    delayed: &[triggered(THIS_ATTACKS, attack_draw_then_may_put_lands_tapped, GRANTED)],
    statics: &[static_ability(
        Layer::Ability,
        attached_object,
        grant_delayed_ability,
        "Equipped creature has \"Whenever this creature attacks, draw X cards, then you may put up to X land cards from your hand onto the battlefield tapped, where X is the number of other creatures you control with a Communicator attached to them.\"",
    )],
    ..oracle::STARFLEET_COMMUNICATOR
};
