use cantrip_api::*;
use cantrip_mechanics::attach::attached_object;
use cantrip_mechanics::statics::grant_delayed_ability;
use cantrip_mechanics::triggers::{THIS_ENTERS, ward};
use cantrip_mechanics::{exile_target_until_leaves_effect, targets};
use cantrip_oracle as oracle;

/// Enchant artifact you control; "exile target creature an opponent controls until this Aura leaves the battlefield" (CR
/// 610.3, nothing if the Aura is already gone, 610.3b); "enchanted permanent has ward {1}" (CR 702.21a).
pub static CARD: CardDef = CardDef {
    enchant: Some(targets::ARTIFACT_YOU_CONTROL),
    abilities: &[triggered(
        THIS_ENTERS,
        exile_target_until_leaves_effect,
        "When this Aura enters, exile target creature an opponent controls until this Aura leaves the battlefield.",
    )
    .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS])],
    delayed: &[ward::<1>("Ward {1} (Whenever this permanent becomes the target of a spell or ability an opponent controls, counter it unless that player pays {1}.)")],
    statics: &[static_ability(Layer::Ability, attached_object, grant_delayed_ability, "Enchanted permanent has ward {1}.")],
    ..oracle::HARDLIGHT_CONTAINMENT
};
