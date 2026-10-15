use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Artifact creatures you control": judged on what each object is after the earlier layers (CR 613.1), its controller
/// the same as Tempered Steel's.
fn affects(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    so_far.is(CardType::Artifact) && so_far.is(CardType::Creature) && q.state().obj(affected).controller == q.state().obj(source).controller
}

fn plus_two_plus_two(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 2);
    chars.toughness = chars.toughness.map(|v| v + 2);
}

pub static CARD: CardDef = CardDef {
    statics: &[StaticDef {
        text: "Artifact creatures you control get +2/+2.",
        layer: Layer::PtModify,
        affects,
        apply: plus_two_plus_two,
        ..StaticDef::BLANK
    }],
    ..oracle::TEMPERED_STEEL
};
