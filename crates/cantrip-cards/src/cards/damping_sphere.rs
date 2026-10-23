use cantrip_api::*;
use cantrip_oracle as oracle;

/// "If a land is tapped for two or more mana, ..." (CR 106.12b, 614.1a): a land's mana ability producing two or more mana
/// in all, of any types.
fn two_or_more_from_a_land(q: &dyn Query, _: &Asker, p: &Proposal) -> bool {
    matches!(p, Proposal::ManaProduction(m) if m.total() >= 2 && q.chars(m.permanent).is(CardType::Land))
}

/// "... it produces {C} instead of any other type and amount."
fn one_colorless_instead(_: &dyn Query, _: &Asker, p: &mut Proposal) {
    if let Proposal::ManaProduction(m) = p {
        m.mana = ManaPool::default();
        m.mana.0[Mana::C as usize] = 1;
    }
}

/// "Each spell a player casts costs {1} more to cast for each other spell that player has cast this turn" (CR 601.2f): the
/// spells that player cast this turn, asked of the history, which records a spell at 601.2i, after its cost is
/// determined, so this one is not among them; spells cast before Damping Sphere arrived count (its rulings). A cost change
/// and a mana cost's generic part are `u8`: 255 spells cast by hand in one turn is past practical scale.
fn tax(q: &dyn Query, _: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    match costed {
        Costed::Spell(_) => CostDelta::more(u8::try_from(q.history().casts_by(player).count()).unwrap_or(u8::MAX)),
        _ => CostDelta::NONE,
    }
}

pub static CARD: CardDef = CardDef {
    cost_changes: &[tax],
    replacements: &[replacement(ReplacementKind::ManaProduction, two_or_more_from_a_land, one_colorless_instead, "produces {C} instead")],
    ..oracle::DAMPING_SPHERE
};
