use cantrip_api::*;
use cantrip_mechanics::{counters, hand::discard_chosen_nonland_from_revealed_hand, targets};
use cantrip_oracle as oracle;

/// Target opponent reveals their hand; you choose a nonland card from it; they discard it (Duress's shape). Then put a
/// +1/+1 counter on a creature you control, chosen as the spell resolves rather than targeted (CR 608.2d): a lone
/// creature is taken without asking, and with none the sentence does nothing.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = item.target(0).player() else { return Ok(()) };
    discard_chosen_nonland_from_revealed_hand(g, item, opponent)?;
    counters::put_on_creature_you_control(g, item, CounterKind::PlusOne, 1)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::OPPONENT], effect),
    ..oracle::HUMILIATE
};
