use cantrip_api::*;
use cantrip_mechanics::{exile_target, targets};
use cantrip_oracle as oracle;

/// "Exile target creature or planeswalker" (CR 701.13a); nothing if it is no longer a legal target (CR 608.2b), the sacrifice paid
/// all the same.
fn exile_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_OR_PLANESWALKER], exile_it),
    // As an additional cost to cast this spell, sacrifice a creature (CR 118.8, 601.2h): exactly one.
    additional_cost: Some(Cost::sacrifice(1, targets::filters::creature)),
    ..oracle::WORTHY_COST
};
