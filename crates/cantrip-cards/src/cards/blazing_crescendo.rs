use cantrip_api::*;
use cantrip_mechanics::library::exile_top_play_until_end_of_next_turn;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// "Target creature gets +3/+1 until end of turn. Exile the top card of your library. Until the end of your
/// next turn, you may play that card." An illegal target prevents the exile too (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    pump(g, item.object, creature, 3, 1);
    exile_top_play_until_end_of_next_turn(g, item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    ..oracle::BLAZING_CRESCENDO
};
