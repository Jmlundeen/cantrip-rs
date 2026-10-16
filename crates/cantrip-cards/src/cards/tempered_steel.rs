use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Artifact creatures you control": judged on what each object is after the earlier layers (CR 613.1), its controller
/// the same as Tempered Steel's.
fn affects(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    so_far.is(CardType::Artifact) && so_far.is(CardType::Creature) && q.state().obj(affected).controller == q.state().obj(source).controller
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::PtModify, affects, cantrip_mechanics::statics::plus_pt::<2, 2>, "Artifact creatures you control get +2/+2.")],
    ..oracle::TEMPERED_STEEL
};
