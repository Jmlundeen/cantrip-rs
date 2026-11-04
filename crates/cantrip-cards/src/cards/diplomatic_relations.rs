use cantrip_api::*;
use cantrip_mechanics::{effects, fight, pump, targets};
use cantrip_oracle as oracle;

/// "Target creature you control gets +1/+0 and gains vigilance until end of turn. It deals damage equal to its power to target
/// creature an opponent controls." (CR 608.2b, 608.2c): the pump and vigilance come first, so the damage is the power it has
/// now; with either target illegal the creature deals no damage, and a spell whose every target is illegal does not resolve.
fn diplomatic_relations(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    instruction(g, |g| {
        pump(g, item.object, creature, 1, 0);
        effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Vigilance);
    });
    fight::targets_bite(g, item, 0, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL, targets::CREATURE_AN_OPPONENT_CONTROLS], diplomatic_relations),
    ..oracle::DIPLOMATIC_RELATIONS
};
