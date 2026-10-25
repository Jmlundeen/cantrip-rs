use cantrip_api::*;
use cantrip_mechanics::{library::exile_top_play_until_end_of_next_turn, targets};
use cantrip_oracle as oracle;

/// Chapter I: "Exile the top four cards of your library. Until the end of your next turn, you may play those cards."
/// The four leave together, face up (CR 101.4); each that reaches exile may be played, cast or as the land for the
/// turn (CR 305.1), until the cleanup step of its controller's next turn (CR 514.2). A library of fewer exiles what it has.
fn exile_four(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_top_play_until_end_of_next_turn(g, item.controller, 4);
    Ok(())
}

/// Chapters II and III: each makes its own delayed ability for the turn (CR 603.7b), so two chapters in one turn
/// trigger twice for each spell.
fn rally(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::delayed::this_turn(g, item, item.object);
    Ok(())
}

fn counter_on_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::plus_one_counter_on_target(g, item)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        chapter(1, exile_four, "I \u{2014} Exile the top four cards of your library. Until the end of your next turn, you may play those cards."),
        chapter(2, rally, "II \u{2014} Whenever you cast a spell this turn, put a +1/+1 counter on target creature you control."),
        chapter(3, rally, "III \u{2014} Whenever you cast a spell this turn, put a +1/+1 counter on target creature you control."),
    ],
    delayed: &[triggered(cantrip_mechanics::triggers::YOU_CAST_A_SPELL, counter_on_creature, "Whenever you cast a spell this turn, put a +1/+1 counter on target creature you control.")
        .targets(&[targets::CREATURE_YOU_CONTROL])],
    ..oracle::SHOWDOWN_OF_THE_SKALDS
};
