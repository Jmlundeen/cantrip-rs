use cantrip_api::*;
use cantrip_mechanics::{effects, firebending};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        firebending::trigger::<1>("Firebending 1 (Whenever this creature attacks, add {R}. This mana lasts until end of combat.)"),
        ability(cost("{1}{R}{R}"), effects::plus_one_counter_on_this, "{1}{R}{R}: Put a +1/+1 counter on this creature."),
    ],
    ..oracle::FIRE_SAGES
};
