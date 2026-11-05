use cantrip_api::*;
use cantrip_mechanics::{delayed, tokens, triggers};
use cantrip_oracle as oracle;

/// "Until end of turn, whenever a creature deals combat damage to you, destroy that creature and create a 2/1 red Klingon
/// Warrior creature token with haste" (CR 603.7b): a delayed triggered ability that lasts the turn, controlled by the
/// spell's controller, so "you" is that player wherever the damage is dealt (CR 603.7d).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    delayed::this_turn(g, item, item.object);
    Ok(())
}

/// "Destroy that creature and create a 2/1 red Klingon Warrior creature token with haste" (CR 701.8a, 701.7): the
/// creature that dealt the damage is the trigger's subject; the token is made even if that creature is gone.
fn destroy_and_create(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(creature) = item.subject {
        g.destroy(creature);
    }
    tokens::create(g, item.controller, &tokens::KLINGON, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    delayed: &[triggered(
        triggers::CREATURE_DEALS_COMBAT_DAMAGE_TO_YOU,
        destroy_and_create,
        "Whenever a creature deals combat damage to you, destroy that creature and create a 2/1 red Klingon Warrior creature token with haste.",
    )],
    ..oracle::RIGHT_OF_VENGEANCE
};

pub static EXTRA: &[&CardDef] = &[&tokens::KLINGON];
