use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target creature card in your graveyard with mana value X" (CR 107.3a, 202.3). X is announced before the target is
/// chosen (CR 601.2b-c by way of 602.2b: `targets_after_x`) and the activation is on the stack by then (CR 602.2a), so the
/// X is that of this source's copy ability topmost on the stack: the one being announced, or the one resolving when its
/// target is judged again (CR 608.2b). Before any is announced every creature card there is allowed, since some X would
/// allow it. A card with {X} in its mana cost has X of 0 there (the ruling, CR 107.3).
fn creature_card_with_mana_value_x(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let Target::Object(card) = t else { return false };
    if !targets::card_in_your_graveyard(q, asker, t, &[CardType::Creature]) {
        return false;
    }
    let announced = q.state().stack.iter().rev().find(|i| {
        i.kind == StackKind::Activated && i.object == asker.source && q.activated_ability(i.ability).is_some_and(|d| d.late_targets)
    });
    announced.is_none_or(|item| q.mana_value(card) == item.x as u32)
}

/// "{X}: This creature becomes a copy of target creature card in your graveyard with mana value X, except it has flying
/// and this ability." The card's printed values (its ruling: nothing of the object it was before), with flying and this
/// very ability added as exceptions, so they are copiable values too (CR 707.9a-b). The effect states no duration, so it
/// lasts as long as the permanent stays on the battlefield (CR 611.2a-b); counters and other effects on it stay (the
/// ruling). It doesn't enter, so the copied card's "enters" abilities don't apply. Nothing if the target is gone
/// (CR 608.2b: the core doesn't resolve it).
fn become_copy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let mut values = g.copiable_values(card);
    values.extra_keywords = values.extra_keywords.with(Keyword::Flying);
    // A card in a graveyard is no copy, so its copiable values carry no exceptions of their own yet.
    values.extra_abilities[0] = Some(item.ability);
    g.become_copy(item.object, item.object, values, Duration::Indefinite);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(TAP, effects::loot, "{T}: Draw a card, then discard a card."),
        ability(cost("{X}"), become_copy, "{X}: This creature becomes a copy of target creature card in your graveyard with mana value X, except it has flying and this ability. Activate only as a sorcery.")
            .targets(&[target(creature_card_with_mana_value_x)])
            .targets_after_x()
            .sorcery_speed(),
    ],
    ..oracle::LIKENESS_LOOTER
};
