use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_mechanics::statics::{other_of_subtype_you_control, plus_pt};
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

/// "Other Cats you control get +1/+1 and have lifelink." One static ability in two layers (CR 613.6): lifelink in layer 6
/// (CR 613.1f, 702.15), +1/+1 in layer 7c (CR 613.4c), to the same objects: every other Cat its controller controls, the
/// type read through the words of this card's text (CR 612).
const TEXT: &str = "Other Cats you control get +1/+1 and have lifelink.";

fn apply(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Lifelink);
}

/// "When this creature enters, create two 1/1 white Cat creature tokens with lifelink." (CR 603.1, 111.1).
fn create_two_1_1_white(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::CAT_LIFELINK, 2)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        create_two_1_1_white,
        "When this creature enters, create two 1/1 white Cat creature tokens with lifelink.",
    )],
    statics: &[
        static_ability(Layer::Ability, other_of_subtype_you_control::<{ lands::sub(Subtype::Cat) }>, apply, TEXT),
        static_ability(Layer::PtModify, other_of_subtype_you_control::<{ lands::sub(Subtype::Cat) }>, plus_pt::<1, 1>, TEXT)
            .continues(),
    ],
    ..oracle::REGAL_CARACAL
};

pub static EXTRA: &[&CardDef] = &[&tokens::CAT_LIFELINK];
