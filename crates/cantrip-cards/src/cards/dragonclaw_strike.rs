use cantrip_api::*;
use cantrip_mechanics::{double, fight, targets};
use cantrip_oracle as oracle;

/// "Double the power and toughness of target creature you control until end of turn. Then it fights up to one target creature an
/// opponent controls" (CR 701.10a-b, 701.14a-b, 608.2b-c): the power and toughness it has as the spell resolves, read once and
/// fixed, then the fight with the doubled power. A first target no longer legal is neither doubled nor fights; with no second
/// target, or only an illegal one, it is doubled and fights nothing.
fn double_then_fight(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    double::power_and_toughness(g, item.object, creature);
    fight::targets_fight(g, item, 0, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL, up_to_one(targets::CREATURE_AN_OPPONENT_CONTROLS.filter)], double_then_fight),
    ..oracle::DRAGONCLAW_STRIKE
};
