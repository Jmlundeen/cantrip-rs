use cantrip_api::*;
use cantrip_mechanics::{effects, land_until_cast};
use cantrip_mechanics::triggers::{CAST_THIS_SPELL, ward_sacrifice};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(CAST_THIS_SPELL, effects::untap_all_lands_you_control, "When you cast this spell, untap all lands you control."),
        ward_sacrifice::<3>("Ward\u{2014}Sacrifice three permanents."),
        land_until_cast::ability::<0>(
            cost("{3}"),
            "{3}, Exile this card from your hand: Target land gains \"{T}: Add {C}{C}\" until this card is cast from exile. You may cast this card for as long as it remains exiled.",
        ),
    ],
    delayed: &[mana_ability(TAP, add_n(Mana::C, 2), "{T}: Add {C}{C}.")],
    ..oracle::EMRAKUL_THE_EXIGENT_DOOM
};
