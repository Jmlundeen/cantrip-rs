use cantrip_api::*;
use cantrip_mechanics::statics::{base_mana_value, creature_too};
use cantrip_mechanics::{attach, targets};
use cantrip_oracle as oracle;

const TEXT: &str = "As long as enchanted artifact isn't a creature, it's an artifact creature with power and toughness each equal to its mana value.";

/// The layer 4 part applies only if the attached artifact is not yet a creature (CR 613.1d).
fn enchanted_noncreature(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    attach::attached_object(q, source, affected, so_far) && !so_far.is(CardType::Creature)
}

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(Layer::Type, enchanted_noncreature, creature_too, TEXT),
        // The same effect continues in layer 7b even after its first part made the artifact a creature (CR 613.6).
        static_ability_for_each(Layer::PtSet, enchanted_noncreature, base_mana_value, TEXT).continues(),
    ],
    enchant: Some(targets::ARTIFACT),
    ..oracle::ANIMATE_ARTIFACT
};
