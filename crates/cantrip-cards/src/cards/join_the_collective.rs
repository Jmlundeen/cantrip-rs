use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "Destroy all creatures. Create X 1/1 colorless Borg artifact creature tokens, where X is the number of nonartifact
/// creatures destroyed this way" (CR 701.8, 111.10b, 608.2c): "destroyed this way" is what the destruction actually
/// removed, so a creature that survives it - indestructible (CR 702.12b), or regenerated as the destruction is applied -
/// is not counted, and an artifact creature that dies is not counted either. The tokens are made after they are all gone.
fn destroy_all_and_make_borg(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.chars(id).is(CardType::Creature))
        .collect();
    // X is the number of nonartifact creatures *destroyed this way*, so it is read off the list the destruction returns: a
    // creature it did not move - indestructible (CR 702.12b), or regenerated as the destruction was applied - is not in it.
    let destroyed = g.destroy_all(&creatures);
    // Each destroyed card is judged by its own types; artifact-ness is the card's, not the battlefield's, so reading it in
    // the graveyard gives the same answer the creature gave as it died.
    let count = destroyed.iter().filter(|&&id| !g.chars(id).is(CardType::Artifact)).count();
    let made = u32::try_from(count).unwrap_or(u32::MAX);
    // `create` with 0 makes none, so the count needs no guard of its own.
    tokens::create(g, item.controller, &tokens::BORG, made)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], destroy_all_and_make_borg),
    ..oracle::JOIN_THE_COLLECTIVE
};

pub static EXTRA: &[&CardDef] = &[&tokens::BORG];
