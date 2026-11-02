use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target opponent discards two cards." (CR 701.9a): their choice.
pub static OBLIVION: CardDef = CardDef {
    spell: spell(&[targets::OPPONENT], effects::target_player_discards::<2>),
    ..oracle::OBLIVION
};

/// "Return target nonland permanent to its owner's hand." (CR 400.3).
pub static CONSIGN: CardDef = CardDef {
    spell: spell(&[targets::NONLAND_PERMANENT], effects::return_first_target_to_hand),
    ..oracle::CONSIGN
};

pub static CARD: CardDef = CardDef {
    split: Some((&CONSIGN, &OBLIVION)),
    ..oracle::CONSIGN_OBLIVION
};

pub static EXTRA: &[&CardDef] = &[&CONSIGN, &OBLIVION];
