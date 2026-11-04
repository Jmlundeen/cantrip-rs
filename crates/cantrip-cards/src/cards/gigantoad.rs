use cantrip_api::*;
use cantrip_mechanics::lands::count_you_control;
use cantrip_mechanics::statics::plus_pt;
use cantrip_oracle as oracle;

/// "As long as you control seven or more lands" (CR 604.1, 611.3a) as the static's `affects`: only the source itself, and only
/// while its controller controls seven or more lands, each judged as the earlier layers make it and by control, not ownership.
fn itself_with_seven_or_more_lands(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    if source != affected {
        return false;
    }
    count_you_control(q, source, &|c| c.is(CardType::Land)) >= 7
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        itself_with_seven_or_more_lands,
        plus_pt::<2, 2>,
        "As long as you control seven or more lands, this creature gets +2/+2.",
    )],
    ..oracle::GIGANTOAD
};
