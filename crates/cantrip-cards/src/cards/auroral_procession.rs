use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CARD_IN_YOUR_GRAVEYARD], cantrip_mechanics::effects::return_first_target_to_hand),
    ..oracle::AURORAL_PROCESSION
};
