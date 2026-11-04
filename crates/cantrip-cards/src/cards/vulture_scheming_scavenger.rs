use cantrip_api::*;
use cantrip_mechanics::effects::other_of_subtype_you_control_gain_until_end_of_turn;
use cantrip_mechanics::lands;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_oracle as oracle;

/// Flying is a core keyword. "Whenever Vulture attacks, other Villains you control gain flying until end of turn." (CR 508.1m, 611.2)
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ATTACKS,
        |g, item| {
            other_of_subtype_you_control_gain_until_end_of_turn::<{ lands::sub(Subtype::Villain) }, { Keywords::of(&[Keyword::Flying]).0 }>(g, item)
        },
        "Whenever Vulture attacks, other Villains you control gain flying until end of turn.",
    )],
    ..oracle::VULTURE_SCHEMING_SCAVENGER
};
