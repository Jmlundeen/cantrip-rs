use cantrip_api::*;
use cantrip_mechanics::{effects, mobilize, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When this creature enters, surveil 1." The surveil action looks at the top card and puts it on top or into the controller's
/// graveyard (CR 603.6a, 701.25a).
pub static CARD: CardDef = CardDef {
    abilities: &[
        mobilize::trigger::<1>("Mobilize 1 (Whenever this creature attacks, create a tapped and attacking 1/1 red Warrior creature token. Sacrifice it at the beginning of the next end step.)"),
        triggered(THIS_ENTERS, |g, item| effects::surveil(g, item, 1), "When this creature enters, surveil 1."),
    ],
    delayed: &[mobilize::SACRIFICE],
    ..oracle::NIGHTBLADE_BRIGADE
};

pub static EXTRA: &[&CardDef] = &[&mobilize::WARRIOR];
