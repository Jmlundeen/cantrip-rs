use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

const DRAW: Question = Question::new(
    "DrawForYou",
    "have that player draw a card?",
    "Otherwise you lose 4 life and that player surveils 1",
    "Draw",
    "Don't",
);

/// "Target opponent may have you draw a card. If they don't, they lose 4 life and you surveil 1."
/// The opponent chooses; the life loss and surveil happen only on declining (CR 118.12, 119.3, 701.25).
fn enters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = targets::player(g, item, 0) else { return Ok(()) };
    if effects::ask_may(g, opponent, item.object, Purpose::Ask(&DRAW))? {
        g.draw(item.controller, 1);
    } else {
        g.lose_life(opponent, 4);
        g.end_sentence();
        effects::surveil(g, item, 1)?;
    }
    Ok(())
}

/// "Whenever you face a dilemma, you may return this enchantment to its owner's hand." A chosen mode is one dilemma;
/// the optional return is a separate triggered ability and returns the source only if it is still on the battlefield.
fn return_on_dilemma(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    static RETURN: Question = Question::new("ReturnThis", "return this enchantment to its owner's hand?", "", "Return", "Keep");
    if effects::ask_may(g, item.controller, item.object, Purpose::Ask(&RETURN))? {
        g.return_to_hand(&[item.object]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, enters, "When this enchantment enters, target opponent may have you draw a card. If they don't, they lose 4 life and you surveil 1.").targets(&[targets::OPPONENT]),
        triggered(triggers::YOU_FACE_A_DILEMMA, return_on_dilemma, "Whenever you face a dilemma, you may return this enchantment to its owner's hand."),
    ],
    ..oracle::TELL_ME_WHAT_YOU_SEE
};
