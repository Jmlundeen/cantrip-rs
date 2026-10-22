use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Target creature gains trample and haste until end of turn. Draw a card. A spell whose only target has become illegal
/// does nothing at all (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    instruction(g, |g| {
        cantrip_mechanics::trample_until_end_of_turn(g, item.object, creature);
        cantrip_mechanics::haste_until_end_of_turn(g, item.object, creature);
    });
    g.end_sentence();
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    ..oracle::IMPOLITE_ENTRANCE
};
