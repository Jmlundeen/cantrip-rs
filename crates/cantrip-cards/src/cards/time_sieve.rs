use cantrip_api::*;
use cantrip_oracle as oracle;

/// Take an extra turn after this one (CR 500.7).
fn extra_turn(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_extra_turns(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP.and(sacrifice(5, cantrip_mechanics::targets::filters::artifact)), extra_turn, "{T}, Sacrifice five artifacts: Take an extra turn after this one.")],
    ..oracle::TIME_SIEVE
};
