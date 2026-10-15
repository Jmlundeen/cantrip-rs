use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Counter target spell unless its controller pays {1}; {4} instead if you control a creature with flying as this
/// resolves (CR 118.12, 608.2c). The spell's controller is asked, and may make mana for it while asked.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if targets::object(g, item, 0).is_none() {
        return Ok(());
    }
    let flyer = g
        .permanents_of(item.controller)
        .into_iter()
        .any(|id| {
            let c = g.chars(id);
            c.is(CardType::Creature) && c.has(Keyword::Flying)
        });
    let price = if flyer { 4 } else { 1 };
    cantrip_mechanics::counter_unless_pays(g, item, 0, mana(price, 0, 0, 0, 0, 0))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::SPELL], effect }),
    ..oracle::LOFTY_DENIAL
};
