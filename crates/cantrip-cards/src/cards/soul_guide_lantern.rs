use cantrip_api::*;
use cantrip_mechanics::{effects, exile_target, targets};
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

fn exile_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    Ok(())
}

/// Exile each opponent's graveyard, every card at once.
fn exile_graveyards(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let mut cards = Vec::new();
    for opponent in g.state().opponents(item.controller).collect::<Vec<_>>() {
        cards.extend_from_slice(g.state().zone(Zone::graveyard(opponent)));
    }
    g.exile_all(&cards);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "{T}, Sacrifice Soul-Guide Lantern: Exile each opponent's graveyard.",
            cost: Cost { tap: true, sacrifice_self: true, ..Cost::NONE },
            effect: exile_graveyards,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{1}, {T}, Sacrifice Soul-Guide Lantern: Draw a card.",
            cost: Cost { mana: mana(1, 0, 0, 0, 0, 0), tap: true, sacrifice_self: true, ..Cost::NONE },
            effect: effects::draw::<1>,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "When Soul-Guide Lantern enters, exile target card from a graveyard.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            targets: &[targets::CARD_IN_A_GRAVEYARD],
            effect: exile_it,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::SOUL_GUIDE_LANTERN
};
