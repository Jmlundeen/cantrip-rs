use cantrip_api::*;
use cantrip_mechanics::{discard_chosen, effects, history, targets, triggers::YOUR_UPKEEP};
use cantrip_oracle as oracle;

/// "This enchantment deals 4 damage to any target and you gain 4 life."
fn damage_and_gain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::damage_target_and_gain_life(g, item, 4, 4)
}

/// "Target opponent discards two cards": their choice (CR 701.9). With its only target illegal the ability does not resolve at
/// all (CR 608.2b), so the target is always there; `targets::player` is the legality check all the same.
fn discard(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(opponent) = targets::player(g, item, 0) {
        discard_chosen(g, opponent, item.object, 2)?;
    }
    Ok(())
}

/// "You lose the game" (CR 104.3a).
fn lose(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.lose_game(item.controller);
    Ok(())
}

/// "Choose one that hasn't been chosen" (CR 700.2): the modes this Pact has chosen before, whoever chose them and whatever
/// became of those abilities, are read from the event history (`history::mode_not_chosen_before`). A mode with targets is
/// offered only while it has a legal one (CR 601.2c), so with only "you lose the game" left it must be chosen, and with no
/// mode left the ability is removed from the stack with no effect (CR 603.3c).
static PACT: ModalDef = modes(&[
    mode(damage_and_gain, "This enchantment deals 4 damage to any target and you gain 4 life.").targets(&[targets::ANY]),
    mode(discard, "Target opponent discards two cards.").targets(&[targets::OPPONENT]),
    mode(|g, item| effects::draw(g, item, 2), "Draw two cards."),
    mode(lose, "You lose the game."),
])
    .allowed(history::mode_not_chosen_before);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(YOUR_UPKEEP, &PACT, "At the beginning of your upkeep, choose one that hasn't been chosen \u{2014} this enchantment deals 4 damage to any target and you gain 4 life; or target opponent discards two cards; or draw two cards; or you lose the game.")],
    ..oracle::DEMONIC_PACT
};
