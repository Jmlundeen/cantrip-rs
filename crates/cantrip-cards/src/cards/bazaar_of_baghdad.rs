use cantrip_api::*;
use cantrip_mechanics::discard_chosen;
use cantrip_oracle as oracle;

/// Draw two cards, then discard three cards (chosen, even when forced; nothing with an empty hand).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 2);
    discard_chosen(g, item.controller, item.object, 3)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP, effect, "{T}: Draw two cards, then discard three cards.")],
    ..oracle::BAZAAR_OF_BAGHDAD
};
