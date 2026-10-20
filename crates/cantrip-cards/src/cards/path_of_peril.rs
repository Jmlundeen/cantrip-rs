use cantrip_api::*;
use cantrip_mechanics::cleave;
use cantrip_oracle as oracle;

/// "Destroy all creatures [with mana value 2 or less]": cleave (CR 702.148a) removes the bracketed words, so when the
/// spell was cast for its cleave cost every creature is destroyed; otherwise only those with mana value 2 or less.
/// They are destroyed all at once (CR 701.8, 101.4).
fn destroy_creatures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cleaved = cleave::cleaved(g, item.object);
    let doomed: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.chars(id).is(CardType::Creature) && (cleaved || g.mana_value(id) <= 2))
        .collect();
    g.destroy_all(&doomed);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], destroy_creatures),
    ways: &[cleave::way(mana("{4}{W}{B}"), "Cleave {4}{W}{B}")],
    ..oracle::PATH_OF_PERIL
};
