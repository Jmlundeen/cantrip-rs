use cantrip_api::*;
use cantrip_mechanics::effects::target_player_loses_1_life_you_gain_1;
use cantrip_mechanics::{targets, warp};
use cantrip_oracle as oracle;

/// "Whenever this creature or another creature or artifact you control dies" (CR 700.4, 603.10a): a creature or an artifact put
/// into a graveyard from the battlefield, looked back on as it last was and under the controller it last had. This creature
/// is a creature, so it is among them.
fn this_or_a_creature_or_artifact_you_control_dies(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. }
            if from.is(ZoneKind::Battlefield)
                && to.is(ZoneKind::Graveyard)
                && q.obj(*old).controller == me.controller
                && (q.chars(*old).is(CardType::Creature) || q.chars(*old).is(CardType::Artifact)) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, this_or_a_creature_or_artifact_you_control_dies),
        target_player_loses_1_life_you_gain_1,
        "Whenever this creature or another creature or artifact you control dies, target opponent loses 1 life and you gain 1 life.",
    )
    .targets(&[targets::OPPONENT])],
    ways: &[warp::way(mana("{B}"), "Warp {B}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::SUSURIAN_VOIDBORN
};
