use cantrip_api::*;
use cantrip_mechanics::statics::{artifacts_you_control, plus_pt};
use cantrip_oracle as oracle;

/// "As long as you control an artifact" (CR 604.1, 611.3a): only the source itself, while its controller controls an artifact (itself
/// counting if it is one), judged as the earlier layers made them and asked again whenever characteristics are computed.
fn itself_while_you_control_an_artifact(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    source == affected && artifacts_you_control(q, source) > 0
}

/// Flying is a core keyword read from its characteristics. "As long as you control an artifact, this creature gets +1/+0" (CR 604.1,
/// 613.4c): a static ability that applies only while its controller controls an artifact.
pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        itself_while_you_control_an_artifact,
        plus_pt::<1, 0>,
        "As long as you control an artifact, this creature gets +1/+0.",
    )],
    ..oracle::CLOUDSCULPT_TECHNICIAN
};
