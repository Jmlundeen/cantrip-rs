use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

static RETURN: Question =
    Question::new("ReturnToBattlefield", "pay {cost} to return {subject} from your graveyard to the battlefield?", "", "Pay {cost}", "Leave it");

/// "You may pay {B}. If you do, return this card from your graveyard to the battlefield."
fn pay_to_return(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::may_pay_to_return_this_to_the_battlefield(g, item, cost("{B}"), &RETURN, &[])
}

pub static CARD: CardDef = CardDef {
    keywords: Keywords::of(&[Keyword::CantBlock]),
    abilities: &[triggered(
        when_asked(EventMask::CRIME, triggers::you_commit_a_crime),
        pay_to_return,
        "Whenever you commit a crime, you may pay {B}. If you do, return this card from your graveyard to the battlefield. (Targeting opponents, anything they control, and/or cards in their graveyards is a crime.)",
    )
    // It returns this card from the graveyard, so it functions only there (CR 113.6m).
    .functions_in(IN_A_GRAVEYARD)],
    ..oracle::FORSAKEN_MINER
};
