use cantrip_api::*;
use cantrip_mechanics::firebending;
use cantrip_mechanics::library::exile_top_card_play_this_turn;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        firebending::trigger::<3>("Firebending 3 (Whenever this creature attacks, add {R}{R}{R}. This mana lasts until end of combat.)"),
        ability(
            cost("{3}"),
            exile_top_card_play_this_turn,
            "{3}: Exile the top card of your library. You may play that card this turn.",
        ),
    ],
    ..oracle::ZUKO_EXILED_PRINCE
};
