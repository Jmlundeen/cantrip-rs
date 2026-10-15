use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_mechanics::library::{Find, search_to_hand};
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

fn equipment(q: &dyn Query, id: ObjectId) -> bool {
    q.chars(id).has_subtype(Subtype::Equipment)
}

/// "You may search your library for an Equipment card, reveal it, put it into your hand, then shuffle." Declining the
/// search ("you may") is choosing no card.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_hand(g, item.controller, item.object, Find::ONE.revealed(), equipment)?;
    Ok(())
}

/// "You may put an Equipment card from your hand onto the battlefield": up to one card; nothing is asked when the hand
/// has none.
fn put_equipment_in(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let hand = g.state().zone(Zone::hand(player)).to_vec();
    let eligible: Vec<ObjectId> = hand.iter().copied().filter(|&c| equipment(g, c)).collect();
    if let Some(&card) = choose::cards_from(g, player, item.object, Purpose::PutOntoBattlefield, hand, eligible, 1, true)?.first() {
        g.put_onto_battlefield(card, player, false)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When Stoneforge Mystic enters, you may search your library for an Equipment card, reveal it, put it into your hand, then shuffle.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: search,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{1}{W}, {T}: You may put an Equipment card from your hand onto the battlefield.",
            cost: Cost { mana: mana(1, 1, 0, 0, 0, 0), tap: true, ..Cost::NONE },
            effect: put_equipment_in,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::STONEFORGE_MYSTIC
};
