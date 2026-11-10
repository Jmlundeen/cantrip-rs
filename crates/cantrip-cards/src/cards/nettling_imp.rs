use cantrip_api::*;
use cantrip_mechanics::combat;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP, combat::compel, "{T}: Choose target non-Wall creature the active player has controlled continuously since the beginning of the turn. That creature attacks this turn if able. Destroy it at the beginning of the next end step if it didn't attack this turn. Activate only during an opponent's turn, before attackers are declared.")
        .targets(&[combat::COMPEL_TARGET]).condition(combat::during_opponents_turn_before_attackers)],
    delayed: &[combat::DESTROY_IT_IF_IT_DIDNT_ATTACK],
    lasting_rules: &[combat::MUST_ATTACK_THIS_TURN],
    ..oracle::NETTLING_IMP
};
