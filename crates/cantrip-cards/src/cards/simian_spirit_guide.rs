use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Exile this card from your hand: Add {R}": a mana ability (CR 605.1a) that functions from the hand (CR 113.6),
/// paid by exiling the card itself.
pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(EXILE_THIS, add(Mana::R), "Exile this card from your hand: Add {R}.")
        .from(ZoneKind::Hand)],
    ..oracle::SIMIAN_SPIRIT_GUIDE
};
