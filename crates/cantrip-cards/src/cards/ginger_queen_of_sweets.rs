use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::{EACH_UPKEEP, THIS_ENTERS};
use cantrip_oracle as oracle;

/// "if you're the monarch" (CR 725.1): an intervening "if" (CR 603.4), checked as the ability triggers and again as it
/// resolves.
fn you_are_the_monarch(q: &dyn Query, asker: &Asker) -> bool {
    q.state().player(asker.controller).monarch
}

/// "create a Gingerbrute token." A token created by name has the characteristics of the Oracle card of that name (CR
/// 111.11, and the card's ruling): Gingerbrute's own definition, its abilities and the block restriction its first one
/// makes among them. The reminder text is no rules text (CR 207.2a), so a text change of Ginger's changes nothing of it.
fn create_gingerbrute(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_by_rule(g, item.controller, &super::gingerbrute::CARD, 1, false)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, effects::become_monarch, "When Ginger enters, you become the monarch."),
        ability(
            cost("{2}, {T}").and(SACRIFICE_THIS),
            |g, item| effects::gain_life(g, item, 6),
            "{2}, {T}, Sacrifice Ginger: You gain 6 life.",
        ),
        triggered(
            EACH_UPKEEP,
            create_gingerbrute,
            "At the beginning of each upkeep, if you're the monarch, create a Gingerbrute token. (It's a {1} 1/1 Food Golem artifact creature with haste, \"{1}: This token can't be blocked this turn except by creatures with haste,\" and \"{2}, {T}, Sacrifice this token: You gain 3 life.\")",
        )
        .condition(you_are_the_monarch),
    ],
    ..oracle::GINGER_QUEEN_OF_SWEETS
};
