use cantrip_api::*;
use cantrip_oracle as oracle;

/// "If a land is tapped for two or more mana, ..." (CR 106.12b, 614.1a): a land's mana ability producing two or more mana
/// in all, of any types.
fn land(q: &dyn Query, _: &Asker, permanent: ObjectId, _: PlayerId) -> bool {
    q.chars(permanent).is(CardType::Land)
}

/// "Each spell a player casts costs {1} more to cast for each other spell that player has cast this turn" (CR 601.2f): the
/// spells that player cast this turn, asked of the history, which records a spell at 601.2i, after its cost is
/// determined, so this one is not among them; spells cast before Damping Sphere arrived count (its rulings). A cost change
/// and a mana cost's generic part are `u8`: 255 spells cast by hand in one turn is past practical scale.
fn tax(q: &dyn Query, _: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    match costed {
        Costed::Spell(_) => CostDelta::more(q.history().casts_by(player).count() as u32),
        _ => CostDelta::NONE,
    }
}

pub static CARD: CardDef = CardDef {
    cost_changes: &[tax],
    replacements: &[fixed_mana_replacement(2, land, Mana::C, 1, "produces {C} instead")],
    ..oracle::DAMPING_SPHERE
};
