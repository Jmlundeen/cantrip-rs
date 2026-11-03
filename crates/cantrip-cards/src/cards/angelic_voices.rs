use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

/// "Creatures you control get +1/+1 as long as you control no nonartifact, nonwhite creatures." (CR 604.1).
fn creatures_control_get_1_1_affects(
    q: &dyn LayerQuery,
    source: ObjectId,
    affected: ObjectId,
    so_far: &Chars,
) -> bool {
    statics::your_creatures(q, source, affected, so_far)
        && !q.state().battlefield().iter().copied().any(|id| {
            let chars = q.chars(id);
            q.controller(id) == q.controller(source)
                && chars.is(CardType::Creature)
                && !chars.is(CardType::Artifact)
                && !chars.colors.has(q.words(source).color(Color::White))
        })
}

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::PtModify,
            creatures_control_get_1_1_affects,
            statics::plus_pt::<1, 1>,
            "Creatures you control get +1/+1 as long as you control no nonartifact, nonwhite creatures.",
        ),
    ],
    ..oracle::ANGELIC_VOICES
};
