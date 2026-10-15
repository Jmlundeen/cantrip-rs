use cantrip_api::*;
use cantrip_oracle as oracle;

/// Every creature its controller controls, itself included.
fn affects(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    so_far.is(CardType::Creature) && q.state().obj(affected).controller == q.state().obj(source).controller
}

fn apply(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Haste);
}

/// "Creatures you control have haste." (CR 702.10, a static ability granting a keyword in layer 6.)
pub static CARD: CardDef = CardDef {
    statics: &[StaticDef {
        text: "Creatures you control have haste.",
        layer: Layer::Ability,
        affects,
        apply,
        ..StaticDef::BLANK
    }],
    ..oracle::SAMUT_HAZORETS_CHAMPION
};
