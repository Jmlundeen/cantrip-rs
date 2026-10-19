use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Target creature with power 3 or less" (CR 115.1, 208.1): its power as the layers give it, judged as it is chosen and again as
/// the spell resolves (CR 608.2b), so a creature that grows past 3 in response is no longer a legal target (the ruling).
fn creature_with_power_3_or_less(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.chars(id).power.is_some_and(|p| p <= 3))
}

/// The Omen (CR 720): "Destroy target creature with power 3 or less. (Then shuffle this card into its owner's library.)" Cast from the
/// hand as an Omen sorcery for {1}{B}; as it resolves it is shuffled into its owner's library (CR 720.3d, the core's).
pub static PETTY_REVENGE: CardDef = CardDef {
    spell: spell(&[target(creature_with_power_3_or_less)], effects::destroy_first_target),
    ..oracle::PETTY_REVENGE
};

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, effects::destroy_first_target, "When this creature enters, destroy up to one target artifact or enchantment.")
        .targets(&[targets::ARTIFACT_OR_ENCHANTMENT.between(0, 1)])],
    omen: Some(&PETTY_REVENGE),
    ..oracle::DISRUPTIVE_STORMBROOD
};

pub static EXTRA: &[&CardDef] = &[&PETTY_REVENGE];
