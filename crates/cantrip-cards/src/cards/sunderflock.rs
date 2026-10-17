use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS_IF_CAST;
use cantrip_oracle as oracle;

/// "This spell costs {X} less to cast, where X is the greatest mana value among Elementals you control" (CR 601.2f, 118.7a):
/// the Elementals on the battlefield, "Elemental" read as this spell's text says it now (CR 612.2), valued as permanents (an {X}
/// in a mana cost is 0, CR 202.3e) and the greatest taken, not their sum. It reduces the generic part of the cost, which
/// never goes below nothing.
fn costs_less_for_the_greatest_elemental(q: &dyn Query, spell: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    // It changes what casting this spell costs, never an ability this card is given.
    let casting = matches!(costed, Costed::Spell(_));
    let elemental = q.words(spell).subtype(Subtype::Elemental);
    let greatest = q
        .permanents_of(player)
        .into_iter()
        .filter(|&id| casting && q.chars(id).has_subtype(elemental))
        .map(|id| q.mana_value(id))
        .max()
        .unwrap_or(0);
    CostDelta::less(greatest.min(255) as u8)
}

/// "Return all non-Elemental creatures to their owners' hands" (CR 400.3, 101.4): every creature on the battlefield that is
/// not an Elemental, whoever controls it, at once; a token that leaves ceases to exist.
fn return_the_non_elementals(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let elemental = g.words(item.object).subtype(Subtype::Elemental);
    let creatures: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| {
            let chars = g.chars(id);
            chars.is(CardType::Creature) && !chars.has_subtype(elemental)
        })
        .collect();
    g.return_to_hand(&creatures);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[costs_less_for_the_greatest_elemental],
    abilities: &[triggered(
        THIS_ENTERS_IF_CAST,
        return_the_non_elementals,
        "When this creature enters, if you cast it, return all non-Elemental creatures to their owners' hands.",
    )],
    ..oracle::SUNDERFLOCK
};
