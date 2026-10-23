use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

const RETURN: Question =
    Question::new("ReturnToBattlefield", "pay {cost} to return {subject} from your graveyard to the battlefield?", "", "Pay {cost}", "Leave it");

/// "You may pay {B}. If you do, return this card from your graveyard to the battlefield." The payment is chosen as the
/// ability resolves; once this card is no longer the object that triggered (CR 400.7) nothing is asked and nothing returns.
fn pay_to_return(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if is_live(g, item.object) && g.may_pay(item.controller, item.object, item.object, cost("{B}"), &RETURN)? {
        g.put_onto_battlefield(item.object, item.controller, false)?;
    }
    Ok(())
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
