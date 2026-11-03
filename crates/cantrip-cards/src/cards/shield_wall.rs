use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Creatures you control get +0/+2 until end of turn." (CR 611.2a, 611.2c, 514.2).
pub static CARD: CardDef = CardDef {
    spell: spell(&[], effects::creatures_you_control_get::<0, 2>),
    ..oracle::SHIELD_WALL
};
