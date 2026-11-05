use cantrip_api::*;
use cantrip_mechanics::combat::power_less_than_its_cant_block_it;
use cantrip_mechanics::statics::plus_pt;
use cantrip_oracle as oracle;

/// "As long as you control a creature with a counter on it" (CR 604.1, 611.3a): only the source itself, while its controller
/// controls a creature (itself counting) with a counter of any kind on it, judged as the earlier layers made the creatures and
/// asked again whenever characteristics are computed.
fn itself_while_you_control_a_creature_with_a_counter(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    let state = q.state();
    let you = state.obj(source).controller;
    source == affected
        && state.battlefield().iter().any(|&id| state.obj(id).controller == you && state.obj(id).has_counters() && q.chars(id).is(CardType::Creature))
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        itself_while_you_control_a_creature_with_a_counter,
        plus_pt::<1, 2>,
        "As long as you control a creature with a counter on it, this creature gets +1/+2.",
    )],
    block_rules: &[power_less_than_its_cant_block_it],
    ..oracle::FORMATION_BREAKER
};
