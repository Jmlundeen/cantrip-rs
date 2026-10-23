use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(TAP.and(SACRIFICE_THIS), effects::exile_first_target_players_graveyard, "{T}, Sacrifice this artifact: Exile target player's graveyard.")
            .targets(&[targets::PLAYER]),
    ],
    ..oracle::TORMODS_CRYPT
};
