use cantrip_api::*;
use cantrip_oracle as oracle;

/// "{4}, {T}: Draw a card. For each mana in this ability's activation cost, you may tap an untapped creature you control
/// rather than pay that mana. Activate only as a sorcery."
pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{4}, {T}: Draw a card. For each mana in this ability's activation cost, you may tap an untapped creature you control rather than pay that mana. Activate only as a sorcery.",
        cost: Cost { mana: mana(4, 0, 0, 0, 0, 0), tap: true, ..Cost::NONE },
        timing: Timing::Sorcery,
        effect: cantrip_mechanics::effects::draw::<1>,
        tap_creatures_for_mana: true,
        ..ActivatedDef::BLANK
    })],
    ..oracle::HEIRLOOM_EPIC
};
