use cantrip_api::*;
use cantrip_mechanics::land_until_cast;
use cantrip_oracle as oracle;

// Vigilance and menace are read from its JSON.
pub static CARD: CardDef = CardDef {
    abilities: &[land_until_cast::ability::<0>(
        cost("{2}"),
        "{2}, Exile this card from your hand: Target land gains \"{T}: Add {B}, {R}, or {G}\" until this card is cast from exile. You may cast this card for as long as it remains exiled.",
    )],
    delayed: &[mana_ability(TAP, add_one_of(&[Color::Black, Color::Red, Color::Green]), "{T}: Add {B}, {R}, or {G}.")],
    ..oracle::MASKED_BANDITS
};
