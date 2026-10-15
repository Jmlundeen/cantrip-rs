use cantrip_api::*;
use cantrip_mechanics::cleave;
use cantrip_oracle as oracle;

/// "Draw a card for each creature you control [with flying]": cleave (CR 702.148a) removes the bracketed words, so
/// when the spell was cast for its cleave cost (way 1, its only alternative cost) every creature counts. Counted as
/// the spell resolves (CR 608.2h).
fn draw_for_creatures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cleaved = cleave::cleaved(g, item.object);
    let count = g
        .permanents_of(item.controller)
        .into_iter()
        .filter(|&id| {
            let chars = g.chars(id);
            chars.types.has(CardType::Creature) && (cleaved || chars.keywords.has(Keyword::Flying))
        })
        .count() as u32;
    if count > 0 {
        g.draw(item.controller, count);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect: draw_for_creatures }),
    ways: &[cleave::way(mana(4, 0, 1, 0, 0, 1), "Cleave {4}{G}{U}")],
    ..oracle::WINGED_PORTENT
};
