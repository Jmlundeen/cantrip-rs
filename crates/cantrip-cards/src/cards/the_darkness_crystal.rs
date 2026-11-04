use cantrip_api::*;
use cantrip_mechanics::{replacements, targets};
use cantrip_oracle as oracle;

/// "Black spells you cast cost {1} less to cast" (CR 601.2f): asked of the spell's own characteristics as it is cast; only its
/// controller's spells, and a generic reduction takes only the generic part (CR 118.7a). "Black" is a word of the Crystal's
/// text (CR 612).
fn black_spells_cost_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let Costed::Spell(spell) = costed else { return CostDelta::NONE };
    if player != q.controller(source) {
        return CostDelta::NONE;
    }
    if q.chars(spell).colors.has(q.words(source).color(Color::Black)) { CostDelta::less(1) } else { CostDelta::NONE }
}

/// "... and you gain 2 life" (CR 614.6): part of the event that replaced the creature's dying, once the card is exiled.
fn gain_two_life(g: &mut dyn Game, asker: &Asker, _: &Proposal, _: &Proposal) {
    g.gain_life(asker.controller, 2);
}

/// "Put target creature card exiled with The Darkness Crystal onto the battlefield tapped under your control with two
/// additional +1/+1 counters on it" (CR 607.2b, 122.6): nothing happens if the card is no longer an exiled card of the
/// Crystal's as the ability resolves (CR 608.2b).
fn put_onto_the_battlefield(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    g.put_onto_battlefield_with_counters(card, item.controller, true, &[(CounterKind::PlusOne, 2)])?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    cost_changes: &[black_spells_cost_less],
    replacements: &[replacement(
        ReplacementKind::ZoneChange,
        replacements::opponents_nontoken_creature_would_die,
        replacements::exile_instead_linked_apply,
        "If a nontoken creature an opponent controls would die, instead exile it and you gain 2 life.",
    )
    .then(gain_two_life)],
    abilities: &[ability(
        cost("{4}{B}{B}, {T}"),
        put_onto_the_battlefield,
        "{4}{B}{B}, {T}: Put target creature card exiled with The Darkness Crystal onto the battlefield tapped under your control with two additional +1/+1 counters on it.",
    )
    .targets(&[targets::CREATURE_CARD_EXILED_WITH_THIS])],
    ..oracle::THE_DARKNESS_CRYSTAL
};
