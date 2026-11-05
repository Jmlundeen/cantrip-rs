use cantrip_api::*;
use cantrip_mechanics::triggers::your_end_step;
use cantrip_mechanics::{choose, each_opponent_sacrifices, effects, history};
use cantrip_oracle as oracle;

const ABZAN: u8 = 0;
const MARDU: u8 = 1;

/// "As this enchantment enters, choose Abzan or Mardu" (CR 614.12a, 607.2f).
fn enters(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    choose::option_as_enters(g, entering, &CARD, 0..2)
}

/// "• Abzan — At the beginning of your end step" (CR 513.1, 603.2): your own end step, and only while Abzan was chosen.
fn abzan_end_step(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    (choose::chosen_option(q, me, &CARD) == Some(ABZAN)).then(|| your_end_step(q, me, event)).flatten()
}

/// "• Mardu — At the beginning of your end step" (CR 513.1, 603.2): your own end step, and only while Mardu was chosen.
fn mardu_end_step(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    (choose::chosen_option(q, me, &CARD) == Some(MARDU)).then(|| your_end_step(q, me, event)).flatten()
}

/// "If a creature died under your control this turn" (CR 603.4, 700.4): asked of the history as the step begins and again as it
/// resolves. Tokens count.
fn a_creature_died_under_your_control_this_turn(q: &dyn Query, asker: &Asker) -> bool {
    history::creatures_died_this_turn(q, Some(asker.controller)) > 0
}

/// "Each opponent sacrifices a creature of their choice" (CR 701.21a, 101.4).
fn edict(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    each_opponent_sacrifices(g, item.controller, item.object, 1, |q, id| q.chars(id).is(CardType::Creature))
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters),
    labels: &["Abzan", "Mardu"],
    abilities: &[
        triggered(
            when_asked(EventMask::STEP_BEGAN, abzan_end_step),
            effects::plus_one_counter_on_each_creature_you_control,
            "Abzan \u{2014} At the beginning of your end step, put a +1/+1 counter on each creature you control.",
        ),
        triggered(
            when_asked(EventMask::STEP_BEGAN, mardu_end_step),
            edict,
            "Mardu \u{2014} At the beginning of your end step, if a creature died under your control this turn, each opponent sacrifices a creature of their choice.",
        )
        .condition(a_creature_died_under_your_control_this_turn),
    ],
    ..oracle::BARRENSTEPPE_SIEGE
};
