use cantrip_api::*;
use cantrip_mechanics::statics::plus_pt;
use cantrip_oracle as oracle;

/// "Other Officers, Artificers, Doctors, Pilots, Scientists, and Spacecraft creatures you control get +2/+2." (CR 613.4c,
/// layer 7c, 109.5, 612.2): every creature of Kirk's controller with one of those subtypes, judged on what the earlier
/// layers made each object and by control, not ownership; "Other" leaves Kirk himself out, and his own subtypes (a Human
/// Officer) would otherwise make him one of them (CR 109.5). Each subtype is read through the words of Kirk's text, so a
/// text-changing effect (CR 612) changes whom he pumps.
fn other_members_of_his_crew(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    if affected == source || !so_far.is(CardType::Creature) || q.controller(affected) != q.controller(source) {
        return false;
    }
    [Subtype::Officer, Subtype::Artificer, Subtype::Doctor, Subtype::Pilot, Subtype::Scientist, Subtype::Spacecraft]
        .into_iter()
        .any(|subtype| so_far.has_subtype(q.words(source).subtype(subtype)))
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        other_members_of_his_crew,
        plus_pt::<2, 2>,
        "Other Officers, Artificers, Doctors, Pilots, Scientists, and Spacecraft creatures you control get +2/+2.",
    )],
    ..oracle::KIRK_MAN_OF_ACTION
};
