use cantrip_api::*;
use cantrip_mechanics::triggers::YOUR_END_STEP;
use cantrip_mechanics::warp;
use cantrip_oracle as oracle;

/// "Whenever Haliya or another creature or artifact you control enters" (CR 603.6a): any creature or artifact that enters
/// under Haliya's controller's control, Haliya included; an artifact creature is one event and triggers once.
fn creature_or_artifact_you_control_enters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield)
                && q.controller(*new) == q.controller(source)
                && (q.chars(*new).is(CardType::Creature) || q.chars(*new).is(CardType::Artifact)) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// You gain 1 life.
fn gain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 1);
    Ok(())
}

/// "Draw a card if you've gained 3 or more life this turn": not an intervening "if" (CR 603.4), so it is asked only as the
/// ability resolves, of the history.
fn draw_if_gained_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.history().life_gained(item.controller) >= 3 {
        g.draw(item.controller, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when(EventMask::ZONE_CHANGE, creature_or_artifact_you_control_enters), gain, "Whenever Haliya or another creature or artifact you control enters, you gain 1 life."),
        triggered(YOUR_END_STEP, draw_if_gained_three, "At the beginning of your end step, draw a card if you've gained 3 or more life this turn."),
    ],
    ways: &[warp::way(mana("{W}"), "Warp {W}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::HALIYA_GUIDED_BY_LIGHT
};
