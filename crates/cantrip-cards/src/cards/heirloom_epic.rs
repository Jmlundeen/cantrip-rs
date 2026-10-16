use cantrip_api::*;
use cantrip_oracle as oracle;

/// "{4}, {T}: Draw a card. For each mana in this ability's activation cost, you may tap an untapped creature you control
/// rather than pay that mana. Activate only as a sorcery."
pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{4}, {T}"), cantrip_mechanics::effects::draw::<1>, "{4}, {T}: Draw a card. For each mana in this ability's activation cost, you may tap an untapped creature you control rather than pay that mana. Activate only as a sorcery.")
        .sorcery_speed()
        .tap_creatures_for_mana()],
    ..oracle::HEIRLOOM_EPIC
};
