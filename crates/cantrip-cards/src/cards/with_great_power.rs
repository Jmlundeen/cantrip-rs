use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, auras_and_equipment_attached_to};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Enchanted creature gets +2/+2 for each Aura and Equipment attached to it" (CR 613.4c, 301.5, 303.4): every Aura and Equipment on the
/// battlefield attached to the creature this Aura is attached to - this Aura among them - judged on what the earlier layers made each
/// permanent (an animated Equipment still counts, and one that stopped being an Equipment does not).
fn plus_two_for_each_aura_and_equipment_attached(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let Some(host) = q.state().obj(source).attached_to else { return };
    let attached = auras_and_equipment_attached_to(q, host);
    let bonus = i32::try_from(2 * attached).unwrap_or(i32::MAX);
    chars.power = chars.power.map(|v| v + bonus);
    chars.toughness = chars.toughness.map(|v| v + bonus);
}

/// "All damage that would be dealt to you" (CR 614.1a, 120.3): damage above zero to this Aura's controller, from any source, combat or not.
fn damage_to_you(_: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::Damage(d) if d.amount > 0 && d.target == Target::Player(asker.controller))
}

/// "... is dealt to enchanted creature instead" (CR 614.9): the same source deals the same damage to the creature this Aura is attached to, and
/// the effects not yet applied are judged against it (CR 616.1f).
fn dealt_to_enchanted_creature_instead(q: &dyn Query, asker: &Asker, proposal: &mut Proposal) {
    if let (Proposal::Damage(d), Some(creature)) = (proposal, q.state().obj(asker.source).attached_to) {
        d.target = Target::Object(creature);
    }
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE_YOU_CONTROL),
    statics: &[static_ability(
        Layer::PtModify,
        attached_object,
        plus_two_for_each_aura_and_equipment_attached,
        "Enchanted creature gets +2/+2 for each Aura and Equipment attached to it.",
    )],
    replacements: &[replacement(
        ReplacementKind::Damage,
        damage_to_you,
        dealt_to_enchanted_creature_instead,
        "the damage is dealt to enchanted creature instead",
    )],
    ..oracle::WITH_GREAT_POWER
};
