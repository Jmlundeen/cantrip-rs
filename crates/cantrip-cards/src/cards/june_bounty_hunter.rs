use cantrip_api::*;
use cantrip_mechanics::statics::{have_cant_be_blocked, itself_while_you_drew_two_or_more};
use cantrip_mechanics::targets::filters;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "Create a Clue token." (CR 111.10f.)
fn make_clue(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::CLUE, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{1}").and(sacrifice(1, filters::another_creature)),
        make_clue,
        "{1}, Sacrifice another creature: Create a Clue token. Activate only during your turn.",
    )
    .timing(Timing::OwnTurn)],
    statics: &[static_ability(
        Layer::Ability,
        itself_while_you_drew_two_or_more,
        have_cant_be_blocked,
        "June can't be blocked as long as you've drawn two or more cards this turn.",
    )],
    ..oracle::JUNE_BOUNTY_HUNTER
};

pub static EXTRA: &[&CardDef] = &[&tokens::CLUE];
