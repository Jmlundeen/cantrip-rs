use cantrip_api::*;
use cantrip_mechanics::mobilize;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mobilize::trigger::<3>("Mobilize 3 (Whenever this creature attacks, create three tapped and attacking 1/1 red Warrior creature tokens. Sacrifice them at the beginning of the next end step.)")],
    delayed: &[mobilize::SACRIFICE],
    ..oracle::DALKOVAN_PACKBEASTS
};

pub static EXTRA: &[&CardDef] = &[&mobilize::WARRIOR];
