use cantrip_api::*;
use cantrip_mechanics::{lands, targets, triggers};
use cantrip_oracle as oracle;

/// "Put a flood counter on target land. It's an Island in addition to its other types for as long as it has a flood
/// counter on it" (CR 122.1, 305.7, 611.2b): the land type is added in layer 4 and keeps the land's others; an Island has
/// "{T}: Add {U}" (CR 305.6). It lasts as long as the counter does, Eluge gone or not (the ruling), and never begins if
/// no flood counter is on it once the counter is put (a replacement that keeps it off, CR 614.1a). Nothing happens if
/// the land is no longer a legal target (CR 608.2b).
fn flood(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(land) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(land, CounterKind::Flood, 1);
    let island = g.words(item.object).subtype(Subtype::Island);
    g.add_effect(item.object, land, EffectKind::AddSubtype(island), Duration::WhileCounter(CounterKind::Flood));
    Ok(())
}

/// "The first instant or sorcery spell you cast each turn costs {U} (or {1}) less to cast for each land you control with
/// a flood counter on it" (CR 601.2f, 118.7d): each {U} of the reduction takes a {U} of the cost, and what the cost has no
/// {U} for takes generic mana instead (the rulings). It is the first such spell of the turn whether or not it was
/// reduced, so it is judged on the history of this turn's casts, any player's turn.
fn first_instant_or_sorcery_costs_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let Costed::Spell(spell) = costed else { return CostDelta::NONE };
    let instant_or_sorcery = |types: TypeSet| types.has(CardType::Instant) || types.has(CardType::Sorcery);
    if player != q.controller(source) || !instant_or_sorcery(q.chars(spell).types) {
        return CostDelta::NONE;
    }
    if q.history().casts_by(player).any(|c| c.spell != spell && instant_or_sorcery(c.types)) {
        return CostDelta::NONE;
    }
    let s = q.state();
    let flooded = s
        .battlefield()
        .iter()
        .filter(|&&id| s.obj(id).controller == player && s.obj(id).counters(CounterKind::Flood) > 0 && q.chars(id).is(CardType::Land))
        .count();
    let mut less = ManaCost::ZERO;
    less.symbols[Mana::U as usize] = flooded as u32;
    CostDelta::less_mana(less)
}

pub static CARD: CardDef = CardDef {
    statics: &[characteristic_defining(Layer::PtDefine, lands::power_and_toughness_are_subtype_you_control::<{ lands::sub(Subtype::Island) }>, "Eluge's power and toughness are each equal to the number of Islands you control.").lost_pt(0, 0)],
    abilities: &[triggered(
        triggers::THIS_ENTERS_OR_ATTACKS,
        flood,
        "Whenever Eluge enters or attacks, put a flood counter on target land. It's an Island in addition to its other types for as long as it has a flood counter on it.",
    )
    .targets(&[targets::LAND])],
    cost_changes: &[first_instant_or_sorcery_costs_less],
    ..oracle::ELUGE_THE_SHORELESS_SEA
};
