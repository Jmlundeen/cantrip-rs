use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target nonlegendary creature" (CR 115.1, 205.4a): a creature on the battlefield without the legendary supertype now.
fn nonlegendary_creature(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && {
        let c = q.chars(id);
        c.is(CardType::Creature) && !c.supertypes.legendary
    })
}

/// "Each other creature becomes a copy of target nonlegendary creature until end of turn." The target's copiable values as
/// the spell resolves (CR 707.2: its printed values or what it is copying, never counters, an animation or other effects;
/// the rulings), applied to every creature on the battlefield then but the target (CR 611.2c: one that becomes a creature or
/// enters later is not affected), tokens included, which stay tokens. Each is the same object, so what else applies to it
/// in the later layers - counters, Auras, pumps - still does (CR 613.1, 613.2a; the ruling), and it may stop being a creature
/// (a copy of an unanimated Mutavault). As the turn ends each is what it was before (CR 514.2). Nothing if the target is no
/// longer legal (CR 608.2b: the core doesn't resolve it).
fn each_other_creature_becomes_a_copy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(original) = targets::object(g, item, 0) else { return Ok(()) };
    let values = g.copiable_values(original);
    let others: Vec<ObjectId> =
        g.state().battlefield().iter().copied().filter(|&id| id != original && g.chars(id).is(CardType::Creature)).collect();
    for creature in others {
        g.become_copy(item.object, creature, values, Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(nonlegendary_creature)], each_other_creature_becomes_a_copy),
    ..oracle::MIRRORWEAVE
};
