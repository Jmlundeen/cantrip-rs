use cantrip_api::*;
use cantrip_mechanics::destroy_target;
use cantrip_oracle as oracle;

/// "Target non-outlaw creature" (CR 115.1). An outlaw is an object with the Assassin, Mercenary, Pirate, Rogue and/or
/// Warlock creature type (CR 700.12): the term, not rules text, so it is not read through `words`.
fn non_outlaw_creature(q: &dyn Query, _: &Asker, t: Target) -> bool {
    const OUTLAW_TYPES: [Subtype; 5] = [Subtype::Assassin, Subtype::Mercenary, Subtype::Pirate, Subtype::Rogue, Subtype::Warlock];
    matches!(t, Target::Object(id) if is_live(q, id) && q.is_on_battlefield(id) && {
        let c = q.chars(id);
        c.is(CardType::Creature) && !OUTLAW_TYPES.iter().any(|&s| c.has_subtype(s))
    })
}

/// "Destroy target non-outlaw creature." (CR 701.8)
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(non_outlaw_creature)], effect),
    ..oracle::SHOOT_THE_SHERIFF
};
