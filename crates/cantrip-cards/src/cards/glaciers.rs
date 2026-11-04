use cantrip_api::*;
use cantrip_mechanics::{effects, lands, statics, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of your upkeep, sacrifice this enchantment unless you pay {W}{U}." (CR 608.2, 701.21a, 118.1).
fn sacrifice_enchantment_unless_pay_w(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::sacrifice_this_unless_pay(g, item, cost("{W}{U}"))
}

/// "All Mountains are Plains." (CR 604.1).
fn mountains(q: &dyn LayerQuery, source: ObjectId, _: ObjectId, so_far: &Chars) -> bool {
    so_far.has_subtype(q.words(source).subtype(Subtype::Mountain))
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::YOUR_UPKEEP,
            sacrifice_enchantment_unless_pay_w,
            "At the beginning of your upkeep, sacrifice this enchantment unless you pay {W}{U}.",
        ),
    ],
    statics: &[
        sets_land_subtypes(
            mountains,
            statics::set_land_type::<{ lands::sub(Subtype::Plains) }>,
            "All Mountains are Plains.",
        ),
    ],
    ..oracle::GLACIERS
};
