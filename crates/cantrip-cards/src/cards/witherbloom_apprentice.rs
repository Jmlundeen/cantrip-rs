use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Magecraft — Whenever you cast or copy an instant or sorcery spell" (CR 207.2c, 601.2i, 707.10): one ability that
/// triggers on either happening. "You" is whoever cast the spell, or whoever made the copy, and the spell read is the
/// object on the stack.
fn cast_or_copy_instant_or_sorcery(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    let (spell, player) = match event {
        Event::SpellCast { spell, player, .. } => (*spell, *player),
        Event::Copied { copy, player, .. } => (*copy, *player),
        _ => return None,
    };
    let c = q.chars(spell);
    (player == q.controller(source) && (c.is(CardType::Instant) || c.is(CardType::Sorcery))).then_some(Fired::PLAIN).flatten()
}

/// Each opponent loses 1 life and you gain 1 life.
fn drain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.each_opponent_loses_life(item.controller, 1);
    g.gain_life(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Magecraft \u{2014} Whenever you cast or copy an instant or sorcery spell, each opponent loses 1 life and you gain 1 life.",
        listens: EventMask(EventMask::SPELL_CAST.0 | EventMask::COPIED.0),
        matches: cast_or_copy_instant_or_sorcery,
        effect: drain,
        ..TriggeredDef::BLANK
    })],
    ..oracle::WITHERBLOOM_APPRENTICE
};
