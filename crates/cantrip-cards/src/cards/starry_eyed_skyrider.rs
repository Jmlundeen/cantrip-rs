use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_mechanics::{effects, statics, targets};
use cantrip_oracle as oracle;

/// "Attacking tokens you control" (CR 506.4, 111.1, 613.1f): an attacking creature of the source's controller, as the earlier
/// layers made it, that is a token; re-asked on every pass, so a token that stops attacking loses what this gives.
fn attacking_tokens_you_control(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    view.state().obj(affected).token && statics::attacking_creatures_you_control(view, source, affected, so_far)
}

// Flying is the core's, read from the card's JSON.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ATTACKS,
        effects::first_target_gains_flying,
        "Whenever this creature attacks, another target creature you control gains flying until end of turn.",
    )
    .targets(&[targets::OTHER_CREATURE_YOU_CONTROL])],
    statics: &[static_ability(Layer::Ability, attacking_tokens_you_control, statics::have_flying, "Attacking tokens you control have flying.")],
    ..oracle::STARRY_EYED_SKYRIDER
};
