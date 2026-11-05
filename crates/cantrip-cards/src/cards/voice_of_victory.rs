use cantrip_api::*;
use cantrip_mechanics::mobilize;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mobilize::trigger::<2>("Mobilize 2 (Whenever this creature attacks, create two tapped and attacking 1/1 red Warrior creature tokens. Sacrifice them at the beginning of the next end step.)")],
    delayed: &[mobilize::SACRIFICE],
    cast_rules: &[statics::opponents_cant_cast_spells_during_your_turn],
    ..oracle::VOICE_OF_VICTORY
};

pub static EXTRA: &[&CardDef] = &[&mobilize::WARRIOR];
