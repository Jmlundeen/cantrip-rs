use cantrip_api::*;
use cantrip_mechanics::library::{look_at_top, Rest};
use cantrip_mechanics::triggers::NONCREATURE_SPELL_YOU_CAST;
use cantrip_mechanics::{delayed, effects, targets};
use cantrip_oracle as oracle;

/// I, II: "Look at the top three cards of your library. Put one of them into your hand and the rest on the bottom of your
/// library in any order." A library with fewer cards shows what it has (CR 609.3), and the one for the hand shrinks with it.
fn look_at_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    look_at_top(g, item.controller, item.object, 3, (1, 1), false, |_| true, Rest::BottomChosen)?;
    Ok(())
}

/// III: "Whenever you cast a noncreature spell this turn, target creature you control gains double strike until end of
/// turn" is a delayed triggered ability for the rest of the turn (CR 603.7b), controlled by the chapter's controller (CR 603.7e).
fn watch_for_noncreature_spells(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    delayed::this_turn(g, item, item.object);
    Ok(())
}

/// "Target creature you control gains double strike until end of turn": nothing if the target is no longer legal (CR 608.2b).
fn double_strike(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    effects::gain_until_end_of_turn(g, item.object, creature, Keyword::DoubleStrike);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        chapter(1, look_at_three, "I, II \u{2014} Look at the top three cards of your library. Put one of them into your hand and the rest on the bottom of your library in any order."),
        chapter(2, look_at_three, "I, II \u{2014} Look at the top three cards of your library. Put one of them into your hand and the rest on the bottom of your library in any order."),
        chapter(3, watch_for_noncreature_spells, "III \u{2014} Whenever you cast a noncreature spell this turn, target creature you control gains double strike until end of turn."),
    ],
    delayed: &[triggered(NONCREATURE_SPELL_YOU_CAST, double_strike, "Whenever you cast a noncreature spell this turn, target creature you control gains double strike until end of turn.")
        .targets(&[targets::CREATURE_YOU_CONTROL])],
    ..oracle::REDISCOVER_THE_WAY
};
