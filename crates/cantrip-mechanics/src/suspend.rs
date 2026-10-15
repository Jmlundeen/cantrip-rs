//! Suspend (CR 702.62a): "Rather than cast this card from your hand, you may pay [cost] and exile it with N time
//! counters on it. At the beginning of your upkeep, remove a time counter from it. When the last is removed, cast it
//! without paying its mana cost." Lotus Bloom and Rift Bolt print it.
//!
//! A card with suspend sets `suspend: Some(SuspendDef { count, cost })` and lists `ability(cost, text)`: the special
//! action (CR 116.2) of exiling it from the hand whenever it could be cast. The two triggered abilities are the rules'
//! own (`cantrip_api::intrinsic`).

use cantrip_api::*;

fn suspend(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let card = item.object;
    let count = g.card(g.obj(card).card).suspend.map_or(0, |s| s.count);
    let exiled = g.exile(card);
    if g.obj(exiled).zone.is(ZoneKind::Exile) {
        g.add_counters(exiled, CounterKind::Time, count as i16);
    }
    Ok(())
}

pub const fn ability(cost: ManaCost, text: &'static str) -> AbilityDef {
    AbilityDef::Activated(ActivatedDef {
        text,
        cost: Cost { mana: cost, ..Cost::NONE },
        timing: Timing::AsCast,
        from: ZoneKind::Hand,
        effect: suspend,
    special_action: true,
        ..ActivatedDef::BLANK
    })
}
