use cantrip_api::*;
use cantrip_mechanics::{lands, statics, storied, tokens, triggers};
use cantrip_oracle as oracle;

pub static EXTRA: &[&CardDef] = &[&tokens::DWARF];

/// "As long as you have an enduring story, creatures you control get +1/+1" (CR 702.195b, 613.4c): every creature its
/// controller controls, itself among them, while that player has the designation.
fn your_creatures_with_an_enduring_story(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    storied::you_have_an_enduring_story(q.state(), source) && statics::your_creatures(q, source, affected, so_far)
}

/// "Create a 2/2 red Dwarf creature token."
fn dwarf(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::DWARF, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    storied: true,
    statics: &[static_ability(Layer::PtModify, your_creatures_with_an_enduring_story, statics::plus_pt::<1, 1>, "As long as you have an enduring story, creatures you control get +1/+1.")],
    abilities: &[triggered(when_asked(EventMask::ZONE_CHANGE, triggers::this_or_another_nontoken_of_subtype_you_control_enters::<{ lands::sub(Subtype::Dwarf) }>), dwarf, "Whenever Fíli or another nontoken Dwarf you control enters, create a 2/2 red Dwarf creature token.")],
    ..oracle::F_LI_THE_PATHFINDER
};
