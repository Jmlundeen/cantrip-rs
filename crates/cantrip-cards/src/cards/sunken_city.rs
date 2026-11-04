use cantrip_api::*;
use cantrip_mechanics::{effects, statics, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of your upkeep, sacrifice this enchantment unless you pay {U}{U}." (CR 608.2, 701.21a, 118.1).
fn sacrifice_enchantment_unless_pay_u(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::sacrifice_this_unless_pay(g, item, cost("{U}{U}"))
}

/// "Blue creatures get +1/+1." (CR 604.1).
fn blue_creatures_get_1_1_affects(q: &dyn LayerQuery, source: ObjectId, _: ObjectId, so_far: &Chars) -> bool {
    so_far.colors.has(q.words(source).color(Color::Blue)) && so_far.is(CardType::Creature)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOUR_UPKEEP,
        sacrifice_enchantment_unless_pay_u,
        "At the beginning of your upkeep, sacrifice this enchantment unless you pay {U}{U}.",
    )],
    statics: &[static_ability(
        Layer::PtModify,
        blue_creatures_get_1_1_affects,
        statics::plus_pt::<1, 1>,
        "Blue creatures get +1/+1.",
    )],
    ..oracle::SUNKEN_CITY
};
