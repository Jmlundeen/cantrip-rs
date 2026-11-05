use cantrip_api::*;
use cantrip_mechanics::{mobilize, statics};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[characteristic_defining(Layer::PtDefine, statics::power_is_creatures_you_control, "Zurgo's Vanguard's power is equal to the number of creatures you control.").lost_pt(0, 3)],
    abilities: &[mobilize::trigger::<1>("Mobilize 1 (Whenever this creature attacks, create a tapped and attacking 1/1 red Warrior creature token. Sacrifice it at the beginning of the next end step.)")],
    delayed: &[mobilize::SACRIFICE],
    ..oracle::ZURGOS_VANGUARD
};

pub static EXTRA: &[&CardDef] = &[&mobilize::WARRIOR];
