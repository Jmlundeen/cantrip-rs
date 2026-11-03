use cantrip_api::*;
use cantrip_mechanics::crew::crew;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

const ASK: Question = Question::new("TapFireNationDrill", "tap The Fire Nation Drill to destroy a creature with power 4 or less?", "", "Tap it", "Don't");

/// "When The Fire Nation Drill enters, you may tap it. When you do, ..." (CR 603.12): tapping is a choice made as the triggered
/// ability resolves, not a cost, and a reflexive trigger is created only if it really became tapped. A Drill that has left the
/// battlefield by now (CR 400.7) or is already tapped cannot be tapped, so nothing is asked. The target of the reflexive trigger is chosen as it goes
/// on the stack.
fn tap_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !is_live(g, item.object) || g.obj(item.object).tapped {
        return Ok(());
    }
    if !effects::ask_may(g, item.controller, item.object, Purpose::Ask(&ASK))? {
        return Ok(());
    }
    g.tap(item.object);
    g.reflexive_trigger(item.object, item.controller, AbilityRef::delayed(g.card_id(&CARD), 0), Target::None);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, tap_it, "When The Fire Nation Drill enters, you may tap it. When you do, destroy target creature with power 4 or less."),
        ability(cost("{1}"), effects::opponents_permanents_lose_hexproof_and_indestructible, "{1}: Permanents your opponents control lose hexproof and indestructible until end of turn."),
        crew(2, "Crew 2 (Tap any number of creatures you control with total power 2 or more: This Vehicle becomes an artifact creature until end of turn.)"),
    ],
    delayed: &[delayed_trigger(effects::destroy_first_target, "When you do, destroy target creature with power 4 or less.")
        .targets(&[target(targets::creature_with_power_at_most::<4>)])],
    ..oracle::THE_FIRE_NATION_DRILL
};
