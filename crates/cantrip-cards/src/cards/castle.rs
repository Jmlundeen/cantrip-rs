use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

/// "Untapped creatures you control get +0/+2." (CR 604.1).
fn untapped_creatures_control_get_0_affects(
    q: &dyn LayerQuery,
    source: ObjectId,
    affected: ObjectId,
    so_far: &Chars,
) -> bool {
    !q.state().obj(affected).tapped && statics::your_creatures(q, source, affected, so_far)
}

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::PtModify,
            untapped_creatures_control_get_0_affects,
            statics::plus_pt::<0, 2>,
            "Untapped creatures you control get +0/+2.",
        ),
    ],
    ..oracle::CASTLE
};
