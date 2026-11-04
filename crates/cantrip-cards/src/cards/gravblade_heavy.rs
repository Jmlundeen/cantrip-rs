use cantrip_api::*;
use cantrip_mechanics::{lands, statics};
use cantrip_oracle as oracle;

const TEXT: &str = "As long as you control an artifact, this creature gets +1/+0 and has deathtouch.";

/// "As long as you control an artifact" (CR 604.1, 613.1f): the source itself, while its controller controls an artifact,
/// each judged as the earlier layers made it. Both of the creature's effects ask it.
fn itself_while_you_control_an_artifact(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    source == affected && lands::count_you_control(q, source, &|c| c.is(CardType::Artifact)) > 0
}

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(Layer::Ability, itself_while_you_control_an_artifact, statics::have_deathtouch, TEXT),
        static_ability(Layer::PtModify, itself_while_you_control_an_artifact, statics::plus_pt::<1, 0>, TEXT),
    ],
    ..oracle::GRAVBLADE_HEAVY
};
