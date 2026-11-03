use cantrip_api::*;
use cantrip_mechanics::statics::plus_pt;
use cantrip_oracle as oracle;

/// "This creature gets +1/+1 as long as there's a Lesson card in your graveyard" (CR 205.3k, 604.1, 611.3a) as the static
/// ability's `affects`: only the source itself, and only while its controller's graveyard holds a Lesson card, each judged as
/// the layers make it, asked again whenever characteristics are computed.
fn itself_with_a_lesson_card_in_your_graveyard(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    let state = q.state();
    source == affected && state.zone(Zone::graveyard(state.obj(source).controller)).iter().any(|&card| q.chars(card).has_subtype(Subtype::Lesson))
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        itself_with_a_lesson_card_in_your_graveyard,
        plus_pt::<1, 1>,
        "This creature gets +1/+1 as long as there's a Lesson card in your graveyard.",
    )],
    ..oracle::FIRST_TIME_FLYER
};
