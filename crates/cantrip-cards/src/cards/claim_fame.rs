use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target creature gets +2/+0 and gains haste until end of turn." (CR 611.2a, 702.10).
pub static FAME: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effects::target_creature_gets_and_gains_haste::<2, 0>),
    ..oracle::FAME
};

/// "Return target creature card with mana value 2 or less from your graveyard to the battlefield." (CR 400.3).
pub static CLAIM: CardDef = CardDef {
    spell: spell(
        &[targets::CREATURE_CARD_WITH_MANA_VALUE_2_OR_LESS_IN_YOUR_GRAVEYARD],
        effects::return_first_target_to_battlefield,
    ),
    ..oracle::CLAIM
};

pub static CARD: CardDef = CardDef {
    split: Some((&CLAIM, &FAME)),
    ..oracle::CLAIM_FAME
};

pub static EXTRA: &[&CardDef] = &[&CLAIM, &FAME];
