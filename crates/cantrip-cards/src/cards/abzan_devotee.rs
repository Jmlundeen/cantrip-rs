use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        // A mana ability (CR 605.1a): it uses no stack and is paid with mana from the pool. "Activate only once each turn"
        // counts the activations this turn (CR 602.5b).
        mana_ability(cost("{1}"), add_one_of(&[Color::White, Color::Black, Color::Green]), "{1}: Add {W}, {B}, or {G}. Activate only once each turn.").once_each_turn(),
        // It functions only in its owner's graveyard (CR 113.6m).
        ability(cost("{2}{B}"), effects::return_this_to_hand, "{2}{B}: Return this card from your graveyard to your hand.").from(ZoneKind::Graveyard),
    ],
    ..oracle::ABZAN_DEVOTEE
};
