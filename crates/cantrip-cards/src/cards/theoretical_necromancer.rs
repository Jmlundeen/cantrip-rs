use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Another target creature card from your graveyard": the exiled card itself is excluded (CR 601.2c, 115.1), while
/// any other creature card there is a legal target even if it was put there this turn.
fn another_creature_card_in_your_graveyard(q: &dyn Query, me: &Asker, t: Target) -> bool {
    (targets::CREATURE_CARD_IN_YOUR_GRAVEYARD.filter)(q, me, t)
        && matches!(t, Target::Object(id) if id != me.source)
}

/// "Return another target creature card from your graveyard to your hand." (CR 400.7: it becomes a new object.)
fn return_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.return_to_hand(&[card]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{3}{B}").and(EXILE_THIS),
        return_it,
        "{3}{B}, Exile this card from your graveyard: Return another target creature card from your graveyard to your hand.",
    )
    .targets(&[target(another_creature_card_in_your_graveyard)])
    .from(ZoneKind::Graveyard)],
    ..oracle::THEORETICAL_NECROMANCER
};
