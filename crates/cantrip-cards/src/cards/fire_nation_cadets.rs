use cantrip_api::*;
use cantrip_mechanics::statics::itself_with_a_lesson_in_your_graveyard;
use cantrip_mechanics::{effects, firebending, statics};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{2}"), effects::this_gets::<1, 0>, "{2}: This creature gets +1/+0 until end of turn.")],
    statics: &[static_ability(
        Layer::Ability,
        itself_with_a_lesson_in_your_graveyard,
        statics::has_delayed_ability::<0>,
        "This creature has firebending 2 as long as there's a Lesson card in your graveyard. (Whenever this creature attacks, add {R}{R}. This mana lasts until end of combat.)",
    )],
    delayed: &[firebending::trigger::<2>("Firebending 2 (Whenever this creature attacks, add {R}{R}. This mana lasts until end of combat.)")],
    ..oracle::FIRE_NATION_CADETS
};
