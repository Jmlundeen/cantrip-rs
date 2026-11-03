use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

static RETURN: Question =
    Question::new("ReturnToBattlefield", "pay {cost} to return {subject} from your graveyard to the battlefield?", "", "Pay {cost}", "Leave it");

/// "You may pay {B}. If you do, return this card from your graveyard to the battlefield with a finality counter on it."
/// (CR 122.6, 614.1c): the counter is there as it enters, so a Wolfbat that would die is exiled instead (CR 122.1h).
fn pay_to_return(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::may_pay_to_return_this_to_the_battlefield(g, item, cost("{B}"), &RETURN, &[(CounterKind::Finality, 1)])
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOU_DRAW_YOUR_SECOND_CARD,
        pay_to_return,
        "Whenever you draw your second card each turn, you may pay {B}. If you do, return this card from your graveyard to the battlefield with a finality counter on it. (If a creature with a finality counter on it would die, exile it instead.)",
    )
    // It returns this card from the graveyard, so it functions only there (CR 113.6m).
    .functions_in(IN_A_GRAVEYARD)],
    ..oracle::WOLFBAT
};
