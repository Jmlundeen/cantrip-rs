use cantrip_api::*;
use cantrip_mechanics::{replacements, triggers::PROWESS};
use cantrip_oracle as oracle;

/// "If a source you control would deal noncombat damage to an opponent or a permanent an opponent controls, it deals that
/// much damage plus 1 instead" (CR 614.1, 120.3): every source its controller controls, Tomik included, while he is on the
/// battlefield.
pub static CARD: CardDef = CardDef {
    abilities: &[PROWESS],
    replacements: &[replacement(
        ReplacementKind::Damage,
        replacements::noncombat_damage_you_deal_to_an_opponent_or_their_permanent,
        replacements::plus::<1>,
        "deals that much damage plus 1 instead",
    )],
    ..oracle::TOMIK_IZZET_SPARKMAGE
};
