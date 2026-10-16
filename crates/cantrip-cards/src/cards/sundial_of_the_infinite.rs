use cantrip_api::*;
use cantrip_mechanics::your_turn;
use cantrip_oracle as oracle;

/// "End the turn." (CR 724.1): the ability is exiled from the stack with every other spell and ability on it (724.1b).
fn end_the_turn(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    g.end_the_turn()
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{1}, {T}"), end_the_turn, "{1}, {T}: End the turn. Activate only during your turn.").condition(your_turn)],
    ..oracle::SUNDIAL_OF_THE_INFINITE
};
