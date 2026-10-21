use cantrip_api::*;
use cantrip_mechanics::{effects, sacrifice_one_of, targets};
use cantrip_oracle as oracle;

/// "Target opponent sacrifices a creature or planeswalker with the greatest mana value among creatures and planeswalkers
/// they control. You gain 2 life." The group is every creature and planeswalker the opponent controls as the spell resolves,
/// judged as one (a planeswalker of mana value 5 outranks a creature of 3); which of those tied for the greatest is
/// sacrificed is their choice (CR 701.21a). The target is the player, so hexproof on a permanent doesn't matter. With no
/// creature or planeswalker nothing is sacrificed, and the life is still gained; if the opponent is no longer a legal target
/// the spell doesn't resolve (CR 608.2b) and nothing happens.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(opponent) = item.target(0) else { return Ok(()) };
    let tied = effects::greatest_mana_value_creature_or_planeswalker(g, opponent);
    sacrifice_one_of(g, opponent, item.object, 1, tied)?;
    g.gain_life(item.controller, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::OPPONENT], effect),
    ..oracle::BREAK_UNDER_PRESSURE
};
