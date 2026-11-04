use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on target creature. Draw a card." With its only target illegal as it resolves the spell does not
/// resolve at all (CR 608.2b), so nothing is drawn.
fn counter_then_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::plus_one_counter_on_target(g, item)?;
    effects::draw(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], counter_then_draw),
    ..oracle::HONOR
};
