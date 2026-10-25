use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::W), "{T}: Add {W}."),
        ability(TAP, cantrip_mechanics::effects::return_first_target_to_hand, "{T}: Return target legendary creature to its owner's hand.")
            .targets(&[targets::LEGENDARY_CREATURE]),
    ],
    ..oracle::KARAKAS
};
