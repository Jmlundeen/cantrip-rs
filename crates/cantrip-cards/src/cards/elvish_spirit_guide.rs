use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// "Exile this card from your hand: Add {G}": a mana ability (CR 605.1a) that functions from the hand (CR 113.6),
/// paid by exiling the card itself.
pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "Exile this card from your hand: Add {G}.",
        cost: Cost { exile_self: true, ..Cost::NONE },
        mana: Some(mana::add_g),
        adds: ManaPool([0, 0, 0, 0, 1, 0]),
        from: ZoneKind::Hand,
        ..ActivatedDef::BLANK
    })],
    ..oracle::ELVISH_SPIRIT_GUIDE
};
