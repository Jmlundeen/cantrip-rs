use cantrip_api::*;
use cantrip_mechanics::{attach, lands, statics, targets};
use cantrip_oracle as oracle;

/// "Enchanted creature has mountainwalk." (CR 604.1, 613.1f, 702.14). Mountain is a word of its text (CR 612.2), so a
/// text-changing effect on this Aura changes the land type it walks.
pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::Ability,
            attach::attached_object,
            statics::have_landwalk::<{ lands::sub(Subtype::Mountain) }>,
            "Enchanted creature has mountainwalk.",
        ),
    ],
    enchant: Some(targets::CREATURE),
    ..oracle::BURROWING
};
