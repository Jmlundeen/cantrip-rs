use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "Target permanent you control gains hexproof until end of turn. Put a +1/+1 counter on it if it's a
/// creature. Put a loyalty counter on it if it's a planeswalker." Check current types on resolution; a creature
/// planeswalker gets both counters.
fn shield_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_effect(
        item.object,
        permanent,
        EffectKind::AddKeywords(Keywords::of(&[Keyword::Hexproof])),
        Duration::EndOfTurn,
    );
    let chars = g.chars(permanent);
    if chars.is(CardType::Creature) {
        g.add_counters(permanent, CounterKind::PlusOne, 1);
    }
    if chars.is(CardType::Planeswalker) {
        g.add_counters(permanent, CounterKind::Loyalty, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        shield_it,
        "When Teyo enters, target permanent you control gains hexproof until end of turn. Put a +1/+1 counter on it if it's a creature. Put a loyalty counter on it if it's a planeswalker.",
    )
    .targets(&[targets::PERMANENT_YOU_CONTROL])],
    ..oracle::TEYO_LIGHTSHIELD_EXPERT
};
