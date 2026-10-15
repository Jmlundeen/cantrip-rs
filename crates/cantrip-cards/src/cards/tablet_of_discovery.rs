use cantrip_api::*;
use cantrip_mechanics::mana::{add_r, tap_for};
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

/// "When this artifact enters, mill a card. You may play that card this turn." (CR 701.17, 601.3, 305.1): the card the
/// mill put into the graveyard is the object played; with an empty library nothing is milled and nothing is granted.
fn mill_and_play(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for card in g.mill(item.controller, 1) {
        g.allow_play(card, item.controller, PlayGrant { lands: true, expiry: Expiry::EndOfTurn, ..PlayGrant::CAST });
    }
    Ok(())
}

/// "{T}: Add {R}{R}. Spend this mana only to cast instant and sorcery spells." (CR 106.6, 605.1a)
fn add_restricted(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    g.add_restricted_mana(asker.controller, Mana::R, 2, ManaRestriction::InstantOrSorcerySpells);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When this artifact enters, mill a card. You may play that card this turn.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: mill_and_play,
            ..TriggeredDef::BLANK
        }),
        tap_for(add_r, Mana::R, "{T}: Add {R}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{T}: Add {R}{R}. Spend this mana only to cast instant and sorcery spells.",
            cost: Cost { tap: true, ..Cost::NONE },
            mana: Some(add_restricted),
            adds: ManaPool([0, 0, 0, 2, 0, 0]),
            restricted: Some(ManaRestriction::InstantOrSorcerySpells),
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::TABLET_OF_DISCOVERY
};
