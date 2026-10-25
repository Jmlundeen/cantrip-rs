use cantrip_api::*;
use cantrip_mechanics::{effects, fight::targets_fight, targets, triggers};
use cantrip_oracle as oracle;

/// "Another target creature you control fights up to one target creature an opponent controls" (CR 701.14a).
/// Neither fights if either target is absent or illegal as this resolves (CR 701.14b, 608.2b).
fn fight(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    targets_fight(g, item, 0, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            fight,
            "When Yoshimaru enters, another target creature you control fights up to one target creature an opponent controls.",
        )
        .targets(&[targets::OTHER_CREATURE_YOU_CONTROL, targets::CREATURE_AN_OPPONENT_CONTROLS.optional()]),
        ability(cost("{6}"), effects::plus_one_counter_on_target, "{6}: Put a +1/+1 counter on target nonlegendary creature.")
            .targets(&[targets::NONLEGENDARY_CREATURE]),
    ],
    ..oracle::YOSHIMARU_SCRAPPY_STRAY
};
