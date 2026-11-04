use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Whenever this creature attacks, mill a card." (CR 701.17a)
fn mill_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.mill(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ATTACKS, mill_a_card, "Whenever this creature attacks, mill a card.")],
    ..oracle::MYSTERIOS_PHANTASM
};
