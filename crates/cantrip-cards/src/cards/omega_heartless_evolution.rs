use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

const FOR_EACH_OPPONENT: TargetSpec = targets::UP_TO_ONE_NONLAND_PERMANENT_OF_EACH_OPPONENT.labelled("Nonland permanent {player} controls to tap and put stun counters on");

/// "... tap up to one target nonland permanent that opponent controls. Put X stun counters on each of those permanents and you
/// gain X life, where X is the number of nonbasic lands you control" (CR 115.1d, 122.1, 119.1): the targets still legal when it
/// resolves are tapped together and each gets X stun counters. X is counted once, as it resolves (the ruling), so the lands that
/// are nonbasic then, as the layers make them, are counted, and the life is gained even if no target is left to stun.
fn tap_stun_and_gain_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let nonbasic_lands = g.permanents_of(you).into_iter().filter(|&id| {
        let c = g.chars(id);
        c.is(CardType::Land) && !c.supertypes.basic
    });
    let x = nonbasic_lands.count() as i32;
    let permanents = targets::legal_objects(g, item);
    g.tap_together(&permanents);
    let puts: Vec<(ObjectId, CounterKind, i32)> = permanents.into_iter().map(|id| (id, CounterKind::Stun, x)).collect();
    g.add_counters_together(&puts);
    effects::gain_life(g, item, x)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        tap_stun_and_gain_life,
        "Wave Cannon \u{2014} When Omega enters, for each opponent, tap up to one target nonland permanent that opponent controls. Put X stun counters on each of those permanents and you gain X life, where X is the number of nonbasic lands you control.",
    )
    .targets(&[FOR_EACH_OPPONENT])],
    ..oracle::OMEGA_HEARTLESS_EVOLUTION
};
