use cantrip_api::*;
use cantrip_mechanics::{effects, targets, tokens, triggers};
use cantrip_oracle as oracle;

/// "Target creature you control with power 2 or less" (CR 115.1, 608.2b): a creature on the battlefield under the activating
/// player's control whose power, as the layers make it, is at most 2, judged again as the ability resolves.
fn creature_you_control_with_power_two_or_less(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE_YOU_CONTROL.filter)(q, asker, t) && targets::creature_with_power_at_most::<2>(q, asker, t)
}

fn create_a_goblin(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::GOBLIN, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, create_a_goblin, "When this creature enters, create a 1/1 red Goblin creature token."),
        ability(
            cost("{1}").and(TAP),
            effects::target_creature_cant_be_blocked_this_turn,
            "{1}, {T}: Target creature you control with power 2 or less can't be blocked this turn.",
        )
        .targets(&[target(creature_you_control_with_power_two_or_less)]),
    ],
    ..oracle::UNDERFOOT_UNDERDOGS
};

pub static EXTRA: &[&CardDef] = &[&tokens::GOBLIN];
