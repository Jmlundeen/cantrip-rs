use cantrip_api::*;
use cantrip_mechanics::{combat, effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(TAP, effects::untap_targets, "{T}: Untap target blue creature.")
            .targets(&[targets::creature_of_color::<{ Color::Blue as u8 }>()]),
        ability(TAP, combat::compel, "{T}: Choose target non-Wall creature the active player has controlled continuously since the beginning of the turn. That creature attacks this turn if able. Destroy it at the beginning of the next end step if it didn't attack this turn. Activate only before attackers are declared.")
            .targets(&[combat::COMPEL_TARGET]).condition(combat::before_attackers_are_declared),
    ],
    delayed: &[combat::DESTROY_IT_IF_IT_DIDNT_ATTACK],
    lasting_rules: &[combat::MUST_ATTACK_THIS_TURN],
    ..oracle::NORRITT
};
