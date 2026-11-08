use cantrip_api::*;
use cantrip_mechanics::library::exile_top_card_play_this_turn;
use cantrip_mechanics::targets::filters;
use cantrip_mechanics::triggers::creature_you_control_deals_combat_damage_to_player;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "Menace" is a keyword in the characteristics. "Whenever one or more creatures you control deal combat damage to a
/// player, create a Treasure token." (CR 510.2, 603.2c, 111.10a): the batch is one trigger for each player damaged, so
/// two attackers hitting the same player make one Treasure and hitting two players makes two. "Sacrifice a Treasure:
/// Exile the top card of your library. You may play that card this turn." (CR 118.3, 601.3): the sacrifice is a cost, so
/// an unpayable one leaves the library alone, and the exiled card is playable only until the turn ends.
pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::DAMAGE, creature_you_control_deals_combat_damage_to_player),
            tokens::create_a_treasure,
            "Whenever one or more creatures you control deal combat damage to a player, create a Treasure token.",
        )
        .sums_per_subject(),
        ability(
            sacrifice(1, filters::treasure),
            exile_top_card_play_this_turn,
            "Sacrifice a Treasure: Exile the top card of your library. You may play that card this turn.",
        ),
    ],
    ..oracle::PROFESSIONAL_FACE_BREAKER
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
