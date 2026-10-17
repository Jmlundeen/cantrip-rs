use cantrip_api::*;
use cantrip_mechanics::hand::discard_chosen_nonland_from_revealed_hand;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Stinging Vitriol deals 2 damage to target opponent. That player reveals their hand. You choose a nonland card from it.
/// They discard that card." With the target illegal as it resolves none of it happens (CR 608.2b). The damage is dealt
/// first (CR 608.2c); the reveal and the choice are asked even when the hand holds no nonland card, as Duress does.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = targets::player(g, item, 0) else { return Ok(()) };
    g.damage(item.object, Target::Player(opponent), 2);
    discard_chosen_nonland_from_revealed_hand(g, item, opponent)?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::OPPONENT], effect), ..oracle::STINGING_VITRIOL };
