use cantrip_api::*;
use cantrip_mechanics::effects::this_gains_deathtouch;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color."),
        ability(cost("{2}"), this_gains_deathtouch, "{2}: This creature gains deathtouch until end of turn."),
    ],
    ..oracle::POISON_DART_FROG
};
