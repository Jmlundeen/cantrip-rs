//! Suspend (CR 702.62a): "Rather than cast this card from your hand, you may pay [cost] and exile it with N time
//! counters on it. At the beginning of your upkeep, remove a time counter from it. When the last is removed, cast it
//! without paying its mana cost." Lotus Bloom and Rift Bolt print it.
//!
//! A card with suspend sets `suspend: Some(SuspendDef { count, cost })` and lists `ability(cost, text)`: the special
//! action (CR 116.2) of exiling it from the hand whenever it could be cast. Printing suspend gives the card the keyword
//! (`Keyword::Suspend`, `CardDef::printed_keywords`); a card that gains suspend (Delay, Epochrasite) is given it by an effect
//! (`EffectKind::AddKeywords`). Either way the layers give an object with the keyword suspend's two triggered abilities,
//! which function in exile (`cantrip_api::intrinsic::SUSPEND_UPKEEP`, `SUSPEND_CAST`), and so is the haste a creature spell
//! cast by the second gains until its caster loses control of it (`intrinsic::SUSPEND_HASTE`, carried to the permanent it
//! becomes, CR 400.7a).

use cantrip_api::*;

fn suspend(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let card = item.object;
    let count = g.card(g.obj(card).card).suspend.map_or(0, |s| s.count);
    let exiled = g.exile(card);
    if g.obj(exiled).zone.is(ZoneKind::Exile) {
        g.add_counters(exiled, CounterKind::Time, count as i32);
    }
    Ok(())
}

pub const fn ability(cost: ManaCost, text: &'static str) -> AbilityDef {
    cantrip_api::ability(Cost::mana(cost), suspend, text)
        .timing(Timing::AsCast)
        .from(ZoneKind::Hand)
        .special_action()
}
