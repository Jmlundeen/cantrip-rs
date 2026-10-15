use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Counter target spell unless its controller pays {3} (CR 118.12). If it is countered this way, exile it instead of
/// putting it into its owner's graveyard.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(spell) = targets::object(g, item, 0) else { return Ok(()) };
    let payer = g.controller(spell);
    if !g.unless_pays(payer, item.object, spell, mana(3, 0, 0, 0, 0, 0))? {
        g.counter_to(spell, CounterTo::Exile);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::SPELL], effect }),
    ..oracle::NO_MORE_LIES
};
