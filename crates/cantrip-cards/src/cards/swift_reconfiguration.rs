use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, crew::crew, statics::grant_delayed_ability, targets};
use cantrip_oracle as oracle;

/// "Enchanted permanent is a Vehicle artifact ... and it loses all other card types." (CR 205.1a, 613.1d, layer 4): its card
/// types become artifact alone and its subtypes the one artifact type Vehicle (the creature types, land types and the like
/// go with the card types they belong to, and Equipment with the other artifact types; its supertypes stay). A later effect
/// in timestamp order may add creature again, as crewing it does (CR 613.7). Vehicle is a word of this card's rules text
/// (CR 612).
fn vehicle_artifact(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    chars.types = TypeSet::of(&[CardType::Artifact]);
    chars.subtypes = SubtypeSet::of(&[view.words(source).subtype(Subtype::Vehicle)]);
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE_OR_VEHICLE),
    statics: &[
        static_ability(Layer::Type, attached_object, vehicle_artifact, "Enchanted permanent is a Vehicle artifact and loses all other card types."),
        static_ability(Layer::Ability, attached_object, grant_delayed_ability, "Enchanted permanent has crew 5."),
    ],
    delayed: &[crew(5, "Crew 5 (Tap any number of creatures you control with total power 5 or more: This Vehicle becomes an artifact creature until end of turn.)")],
    ..oracle::SWIFT_RECONFIGURATION
};
