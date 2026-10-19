use cantrip_api::*;
use cantrip_mechanics::statics::plus_pt;
use cantrip_oracle as oracle;

/// "Other Angels you control get +1/+1 and have lifelink." One static ability in two layers (CR 613.6): lifelink in layer 6
/// (CR 613.1f, 702.15), +1/+1 in layer 7c (CR 613.4c), to the same objects: every other permanent its controller controls that is an
/// Angel as the layers have it so far, the type read through the words of this card's text (CR 612).
const TEXT: &str = "Other Angels you control get +1/+1 and have lifelink.";

fn affects(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    affected != source
        && so_far.has_subtype(q.words(source).subtype(Subtype::Angel))
        && q.state().obj(affected).controller == q.state().obj(source).controller
}

fn apply(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Lifelink);
}

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(Layer::Ability, affects, apply, TEXT),
        static_ability(Layer::PtModify, affects, plus_pt::<1, 1>, TEXT).continues(),
    ],
    ..oracle::LYRA_DAWNBRINGER
};
