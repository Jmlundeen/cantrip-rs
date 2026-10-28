use cantrip_api::*;
use cantrip_mechanics::{effects, statics, triggers};
use cantrip_oracle as oracle;

/// "If an artifact entered the battlefield under your control this turn" (CR 603.4): a permanent that entered this turn under
/// this ability's controller and was an artifact as it entered (CR 603.6a), whatever it is or who controls it now. Asked as it
/// triggers and again as it resolves.
fn an_artifact_entered_under_your_control_this_turn(q: &dyn Query, asker: &Asker) -> bool {
    q.history().entered().any(|e| e.controller == asker.controller && e.is(CardType::Artifact))
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ATTACKS,
        |g, item| effects::draw(g, item, 1),
        "Whenever Iron Man attacks, if an artifact entered the battlefield under your control this turn, draw a card.",
    )
    .condition(an_artifact_entered_under_your_control_this_turn)],
    statics: &[static_ability(
        Layer::PtModify,
        statics::itself,
        statics::plus_power_per_other_artifact_you_control,
        "Iron Man gets +1/+0 for each other artifact you control.",
    )],
    ..oracle::IRON_MAN_MASTER_OF_MACHINES
};
