use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// The Adventure (CR 715): Return target nonland permanent an opponent controls to its owner's hand.
pub static PETTY_THEFT: CardDef = CardDef {
    spell: spell(&[targets::NONLAND_PERMANENT_AN_OPPONENT_CONTROLS], cantrip_mechanics::effects::return_first_target_to_hand),
    ..oracle::PETTY_THEFT
};

pub static CARD: CardDef = CardDef {
    // "This creature can block only creatures with flying."
    keywords: Keywords::of(&[Keyword::Flash, Keyword::Flying, Keyword::CanBlockOnlyCreaturesWithFlying]),
    adventure: Some(&PETTY_THEFT),
    ..oracle::BRAZEN_BORROWER
};

pub static EXTRA: &[&CardDef] = &[&PETTY_THEFT];
