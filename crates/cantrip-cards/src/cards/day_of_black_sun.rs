use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Each creature with mana value X or less loses all abilities until end of turn. Destroy those creatures." The
/// creatures are the ones with mana value X or less as the spell begins to resolve (CR 608.2c), read through the
/// layers (`mana_value`: a copy has the copied card's, a token copy of a creature the same, X is the spell's own X
/// while the spell is on the stack and 0 for a creature on the battlefield, CR 202.3). Each loses all abilities until
/// the cleanup step (CR 613.1f, 611.2a) and then the same creatures are destroyed together (CR 701.8a): losing the
/// ability of indestructible makes them destroyable, while a regeneration shield, which is no ability of theirs, still
/// replaces the destruction.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = item.x as u32;
    let creatures: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.chars(id).is(CardType::Creature) && g.mana_value(id) <= x)
        .collect();
    for &creature in &creatures {
        g.add_effect(item.object, creature, EffectKind::LoseAllAbilities, Duration::EndOfTurn);
    }
    g.destroy_all(&creatures);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    ..oracle::DAY_OF_BLACK_SUN
};
