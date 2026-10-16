use cantrip_api::*;
use cantrip_mechanics::replacements::GAIN_THAT_MUCH_LIFE_PLUS_ONE;
use cantrip_mechanics::statics::plus_pt;
use cantrip_oracle as oracle;

/// "This creature gets +2/+2 as long as you have 25 or more life" (CR 613.4c): itself, while its controller's life total
/// is 25 or more, whatever the starting life total (the 2024-11-08 ruling).
fn itself_at_25_or_more_life(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    let state = view.state();
    affected == source && state.player(state.obj(source).controller).life >= 25
}

pub static CARD: CardDef = CardDef {
    replacements: &[GAIN_THAT_MUCH_LIFE_PLUS_ONE],
    statics: &[static_ability(Layer::PtModify, itself_at_25_or_more_life, plus_pt::<2, 2>, "This creature gets +2/+2 as long as you have 25 or more life.")],
    ..oracle::ANGEL_OF_VITALITY
};
