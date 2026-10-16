use cantrip_api::*;
use cantrip_oracle as oracle;

/// I: "You draw two cards and you lose 2 life."
fn chapter_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 2);
    g.lose_life(item.controller, 2);
    Ok(())
}

/// II, III: the delayed ability that lasts the turn (CR 603.7b), made by the chapter. "You" is the Saga's controller as
/// the chapter resolved (CR 603.7d).
fn watch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let this = g.card_id(&CARD);
    g.delayed_trigger(DelayedWhen::ThisTurn, item.object, item.controller, AbilityRef::delayed(this, 0), Target::None);
    Ok(())
}

/// "Each opponent loses 1 life and you gain 1 life."
fn drain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.each_opponent_loses_life(item.controller, 1);
    g.gain_life(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        chapter(1, chapter_one, "I \u{2014} You draw two cards and you lose 2 life."),
        chapter(2, watch, "II \u{2014} Whenever a creature you control enters this turn, each opponent loses 1 life and you gain 1 life."),
        chapter(3, watch, "III \u{2014} Whenever a creature you control enters this turn, each opponent loses 1 life and you gain 1 life."),
    ],
    delayed: &[triggered(cantrip_mechanics::triggers::CREATURE_YOU_CONTROL_ENTERS, drain, "Whenever a creature you control enters this turn, each opponent loses 1 life and you gain 1 life.")],
    ..oracle::THUNDER_OF_UNITY
};
