use cantrip_api::*;
use cantrip_oracle as oracle;

/// "End the turn." (CR 724.1), then "At the beginning of your next end step, you lose the game.": the spell's later
/// instruction still happens after it is exiled from the stack (724.1b), setting up a delayed triggered ability (CR
/// 603.7) for its controller's next end step, which, with this turn's end step skipped (724.1e), is never this turn's.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.end_the_turn()?;
    let this = g.card_id(&CARD);
    g.delayed_trigger(
        DelayedWhen::Step { step: Step::End, on_turn_of: Some(item.controller) },
        item.object,
        item.controller,
        AbilityRef::delayed(this, 0),
        Target::None,
    );
    Ok(())
}

/// "You lose the game" (CR 104.3e).
fn lose(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.lose_game(item.controller);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    delayed: &[delayed_trigger(lose, "At the beginning of your next end step, you lose the game.")],
    ..oracle::GLORIOUS_END
};
