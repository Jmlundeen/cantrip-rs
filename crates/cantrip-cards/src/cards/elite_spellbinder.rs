use cantrip_api::*;
use cantrip_mechanics::{choose, targets, triggers::this_enters};
use cantrip_oracle as oracle;

/// When this creature enters, look at target opponent's hand. You may exile a nonland card from it. For as long as that
/// card remains exiled, its owner may play it. A spell cast this way costs {2} more to cast (CR 601.2f).
fn look_and_exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = item.targets[0].player() else { return Ok(()) };
    let hand = g.state().zone(Zone::hand(opponent)).to_vec();
    let eligible: Vec<ObjectId> = hand.iter().copied().filter(|&c| !g.chars(c).is(CardType::Land)).collect();
    if let Some(&card) = choose::cards_from(g, item.controller, item.object, Purpose::Exile, hand, eligible, 1, true)?.first() {
        let owner = g.obj(card).owner;
        let exiled = g.exile(card);
        if g.obj(exiled).zone.is(ZoneKind::Exile) && !g.obj(exiled).token {
            g.allow_play(exiled, owner, PlayGrant { lands: true, more: 2, ..PlayGrant::CAST });
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When this creature enters, look at target opponent's hand. You may exile a nonland card from it. For as long as that card remains exiled, its owner may play it. A spell cast this way costs {2} more to cast.",
        listens: EventMask::ZONE_CHANGE,
        matches: this_enters,
        targets: &[targets::OPPONENT],
        effect: look_and_exile,
        ..TriggeredDef::BLANK
    })],
    ..oracle::ELITE_SPELLBINDER
};
