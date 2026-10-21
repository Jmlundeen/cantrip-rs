use cantrip_api::*;
use cantrip_mechanics::{choose, effects, targets};
use cantrip_oracle as oracle;

/// "Target opponent exiles a creature or planeswalker they control with the greatest mana value among creatures and
/// planeswalkers they control." The group is every creature and planeswalker the opponent controls as the spell resolves,
/// judged as one; the opponent chooses which of those tied for the greatest is exiled (CR 608.2d), asked even when one
/// has it, as Strategic Betrayal's exile is. The target is the player, so hexproof on a permanent doesn't matter. With no
/// creature or planeswalker nothing is exiled; if the opponent is no longer a legal target the spell doesn't resolve
/// (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = item.target(0).player() else { return Ok(()) };
    let tied = effects::greatest_mana_value_creature_or_planeswalker(g, opponent);
    if let Some(chosen) = choose::card(g, opponent, item.object, Purpose::Exile, tied)? {
        g.exile(chosen);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::OPPONENT], effect),
    ..oracle::END_OF_THE_HUNT
};
