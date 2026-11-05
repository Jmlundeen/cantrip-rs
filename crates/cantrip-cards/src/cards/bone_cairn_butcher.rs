use cantrip_api::*;
use cantrip_mechanics::{mobilize, statics};
use cantrip_oracle as oracle;

/// "Attacking tokens you control" (CR 111.1, 506.4, 613.1f): an attacking creature under the source's controller's control,
/// as the earlier layers made it, that is a token.
fn attacking_tokens_you_control(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    view.state().obj(affected).token && statics::attacking_creatures_you_control(view, source, affected, so_far)
}

pub static CARD: CardDef = CardDef {
    abilities: &[mobilize::trigger::<2>("Mobilize 2 (Whenever this creature attacks, create two tapped and attacking 1/1 red Warrior creature tokens. Sacrifice them at the beginning of the next end step.)")],
    delayed: &[mobilize::SACRIFICE],
    statics: &[static_ability(
        Layer::Ability,
        attacking_tokens_you_control,
        statics::have_deathtouch,
        "Attacking tokens you control have deathtouch.",
    )],
    ..oracle::BONE_CAIRN_BUTCHER
};

pub static EXTRA: &[&CardDef] = &[&mobilize::WARRIOR];
