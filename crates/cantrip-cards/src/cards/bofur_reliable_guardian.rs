use cantrip_api::*;
use cantrip_mechanics::effects::target_gains_hexproof_and_indestructible_until_end_of_turn as concerted_care;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// The Adventure (CR 715): Concerted Care.
pub static CONCERTED_CARE: CardDef = CardDef {
    spell: spell(&[targets::ARTIFACT_OR_CREATURE_YOU_CONTROL], concerted_care),
    ..oracle::CONCERTED_CARE
};

pub static CARD: CardDef = CardDef { adventure: Some(&CONCERTED_CARE), ..oracle::BOFUR_RELIABLE_GUARDIAN };

pub static EXTRA: &[&CardDef] = &[&CONCERTED_CARE];
