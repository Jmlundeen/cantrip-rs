use cantrip_api::*;
use cantrip_mechanics::triggers::MAGECRAFT;
use cantrip_oracle as oracle;

/// Each opponent loses 1 life and you gain 1 life.
fn drain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.each_opponent_loses_life(item.controller, 1);
    g.gain_life(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(MAGECRAFT, drain, "Magecraft \u{2014} Whenever you cast or copy an instant or sorcery spell, each opponent loses 1 life and you gain 1 life.")],
    ..oracle::WITHERBLOOM_APPRENTICE
};
