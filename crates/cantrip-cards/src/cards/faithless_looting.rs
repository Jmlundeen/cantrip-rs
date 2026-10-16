use cantrip_api::*;
use cantrip_mechanics::flashback;
use cantrip_oracle as oracle;
use cantrip_mechanics::discard_chosen;

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 2);
    discard_chosen(g, item.controller, item.object, 2)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ways: &[flashback::way(cost("{2}{R}"), "Flashback {2}{R}")],
    ..oracle::FAITHLESS_LOOTING
};
