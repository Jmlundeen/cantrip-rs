use cantrip_api::*;
use cantrip_mechanics::mobilize;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mobilize::trigger::<1>("Mobilize 1 (Whenever this creature attacks, create a tapped and attacking 1/1 red Warrior creature token. Sacrifice it at the beginning of the next end step.)")],
    delayed: &[mobilize::SACRIFICE],
    ..oracle::DRAGONBACK_LANCER
};

pub static EXTRA: &[&CardDef] = &[&mobilize::WARRIOR];
