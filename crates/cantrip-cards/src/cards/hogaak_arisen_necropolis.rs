use cantrip_api::*;
use cantrip_oracle as oracle;

// Characteristics only: convoke, delve, "you can't spend mana to cast this spell" and "you may cast this card from your graveyard" are the
// core's: they are characteristics of the card (`pay_with` and the two keywords), so nothing is left to write.
pub static CARD: CardDef = CardDef { ..oracle::HOGAAK_ARISEN_NECROPOLIS };
