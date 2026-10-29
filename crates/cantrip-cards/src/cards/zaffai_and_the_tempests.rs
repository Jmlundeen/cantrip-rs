use cantrip_api::*;
use cantrip_mechanics::history;
use cantrip_oracle as oracle;

/// "Once during each of your turns, you may cast an instant or sorcery spell from your hand without paying its mana cost": an
/// alternative cost this permanent gives (CR 118.9, 601.2b), for no mana (X is 0, CR 107.3b; additional costs are still paid,
/// CR 118.9d), judged on the face being cast - a split card with fuse may be cast fused for it, both halves free (the fuse
/// ruling, CR 702.102a). It is used up by a cast for it this turn, which the history records as paid for by this permanent
/// (`Event::PaidAlternative`); another Zaffai, or this one come back, is another (CR 400.7).
fn once_each_of_your_turns(q: &dyn Query, asker: &Asker, spell: ObjectId, player: PlayerId) -> bool {
    let c = q.chars(spell);
    player == asker.controller
        && q.state().turn.active == player
        && q.obj(spell).zone == Zone::hand(player)
        && (c.is(CardType::Instant) || c.is(CardType::Sorcery))
        && !history::permission_used_this_turn(q, asker.source)
}

pub static CARD: CardDef = CardDef {
    spell_alternatives: &[SpellAlternative {
        text: "Once during each of your turns, you may cast an instant or sorcery spell from your hand without paying its mana cost.",
        applies: once_each_of_your_turns,
        cost: InsteadOfManaCost::WithoutPaying,
        flash: false,
    }],
    ..oracle::ZAFFAI_AND_THE_TEMPESTS
};
