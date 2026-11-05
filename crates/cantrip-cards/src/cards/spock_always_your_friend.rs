use cantrip_api::*;
use cantrip_mechanics::{library, triggers};
use cantrip_oracle as oracle;

/// "When Spock enters, look at the top four cards of your library. You may reveal a noncreature, nonland card from among
/// them and put it into your hand. Put the rest on the bottom of your library in any order." (CR 701.20, 401.4) The look
/// is the instruction and the taking its option: with nothing qualifying they are still looked at and stay on the bottom
/// in the order their owner chooses.
fn look(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let noncreature_nonland = |c: &Chars| !c.is(CardType::Creature) && !c.is(CardType::Land);
    library::look_at_top(g, item.controller, item.object, 4, (0, 1), true, noncreature_nonland, library::Rest::BottomChosen)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        look,
        "When Spock enters, look at the top four cards of your library. You may reveal a noncreature, nonland card from among them and put it into your hand. Put the rest on the bottom of your library in any order.",
    )],
    ..oracle::SPOCK_ALWAYS_YOUR_FRIEND
};
