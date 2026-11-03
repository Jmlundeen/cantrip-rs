use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when(EventMask::TAPPED, cantrip_mechanics::triggers::attached_permanent_becomes_tapped),
            cantrip_mechanics::effects::ping_subject_controller::<2>,
            "Whenever enchanted land becomes tapped, this Aura deals 2 damage to that land's controller.",
        ),
    ],
    enchant: Some(cantrip_mechanics::targets::LAND),
    ..oracle::PSYCHIC_VENOM
};
