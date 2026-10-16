use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, land_until_cast};
use cantrip_oracle as oracle;

// Flying is read from its JSON.
pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, |g, item| effects::gain_life(g, item, 3), "When this creature enters, you gain 3 life."),
        land_until_cast::ability::<0>(
            cost("{2}"),
            "{2}, Exile this card from your hand: Target land gains \"{T}: Add {W}, {U}, or {B}\" until this card is cast from exile. You may cast this card for as long as it remains exiled.",
        ),
    ],
    delayed: &[mana_ability(TAP, add_one_of(&[Color::White, Color::Blue, Color::Black]), "{T}: Add {W}, {U}, or {B}.")],
    ..oracle::SHATTERED_SERAPH
};
