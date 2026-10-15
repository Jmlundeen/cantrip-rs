use cantrip_api::*;
use cantrip_mechanics::attach::{equipment_you_control_enters, may_attach_that_equipment};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "You may cast Aura and Equipment spells as though they had flash" (CR 307.5, 702.8): its controller's, whatever zone
/// they cast them from.
fn aura_or_equipment_you_cast(q: &dyn Query, asker: &Asker, spell: ObjectId, caster: PlayerId) -> bool {
    let chars = q.chars(spell);
    caster == asker.controller && (chars.has_subtype(Subtype::Aura) || chars.has_subtype(Subtype::Equipment))
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever an Equipment you control enters, you may attach it to target creature you control.",
        listens: EventMask::ZONE_CHANGE,
        matches: equipment_you_control_enters,
        targets: &[targets::CREATURE_YOU_CONTROL],
        effect: may_attach_that_equipment,
        ..TriggeredDef::BLANK
    })],
    flash_rules: &[aura_or_equipment_you_cast],
    ..oracle::SIGARDAS_AID
};
