use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

/// "This creature gets +0/+1 as long as an opponent controls an Island." (CR 604.1, 613.1f): only the source itself, the subtype
/// read through the words of its text (CR 612.2).
fn while_opponent_controls_island(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    let island = q.words(source).subtype(Subtype::Island);
    let you = q.controller(source);
    source == affected
        && q.state().battlefield().iter().any(|&id| q.controller(id) != you && q.chars(id).has_subtype(island))
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        while_opponent_controls_island,
        statics::plus_pt::<0, 1>,
        "This creature gets +0/+1 as long as an opponent controls an Island.",
    )],
    ..oracle::WATER_WURM
};
