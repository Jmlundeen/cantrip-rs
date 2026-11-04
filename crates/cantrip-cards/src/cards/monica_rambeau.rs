use cantrip_api::*;
use cantrip_mechanics::triggers::{NONCREATURE_SPELL_YOU_CAST, PROWESS};
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// Photon, Living Light: flying and hexproof are core keywords.
pub static BACK: CardDef = CardDef {
    abilities: &[
        PROWESS,
        triggered(
            NONCREATURE_SPELL_YOU_CAST,
            effects::plus_one_counter_on_each_other_creature_you_control,
            "Whenever you cast a noncreature spell, put a +1/+1 counter on each other creature you control.",
        ),
    ],
    ..oracle::PHOTON_LIVING_LIGHT
};

/// Monica Rambeau: flying is a core keyword. "{2}{R}{W}{W}: Transform Monica Rambeau. Activate only as a sorcery." (CR 701.27a,
/// 712.9): she stays the same object, with Photon's characteristics; an ability of hers does nothing if she has transformed since
/// it was put on the stack (CR 701.27f).
pub static CARD: CardDef = CardDef {
    abilities: &[
        PROWESS,
        ability(cost("{2}{R}{W}{W}"), effects::transform_self, "{2}{R}{W}{W}: Transform Monica Rambeau. Activate only as a sorcery.").sorcery_speed(),
    ],
    back: Some(&BACK),
    ..oracle::MONICA_RAMBEAU
};
