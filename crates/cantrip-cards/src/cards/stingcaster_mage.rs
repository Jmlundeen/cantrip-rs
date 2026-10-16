use cantrip_api::*;
use cantrip_mechanics::{grant_flashback, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, grant_flashback, "When this creature enters, target instant or sorcery card in your graveyard gains flashback until end of turn. The flashback cost is equal to its mana cost.")
        .targets(&[targets::INSTANT_OR_SORCERY_IN_YOUR_GRAVEYARD])],
    ..oracle::STINGCASTER_MAGE
};
