use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::A_PLAYER_CASTS_THEIR_SECOND_SPELL,
        // Any opponent of its controller, not necessarily the player who cast the spell (the ruling).
        effects::damage_first_target::<2>,
        "Whenever a player casts their second spell each turn, this creature deals 2 damage to target opponent.",
    )
    .targets(&[targets::OPPONENT])],
    ..oracle::HEARTHBORN_BATTLER
};
