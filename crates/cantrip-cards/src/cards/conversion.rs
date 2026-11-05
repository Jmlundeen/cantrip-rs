use cantrip_api::*;
use cantrip_mechanics::{
    effects,
    lands::sub,
    statics::{all_of_subtype, set_land_type},
    triggers,
};
use cantrip_oracle as oracle;

/// "At the beginning of your upkeep, sacrifice this enchantment unless you pay {W}{W}." (CR 608.2, 701.21a, 118.1).
fn sacrifice_enchantment_unless_pay_ww(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::sacrifice_this_unless_pay(g, item, cost("{W}{W}"))
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOUR_UPKEEP,
        sacrifice_enchantment_unless_pay_ww,
        "At the beginning of your upkeep, sacrifice this enchantment unless you pay {W}{W}.",
    )],
    // "All Mountains are Plains." (CR 305.7, 613.1d): every Mountain, whoever controls it, becomes only a Plains.
    statics: &[sets_land_subtypes(
        all_of_subtype::<{ sub(Subtype::Mountain) }>,
        set_land_type::<{ sub(Subtype::Plains) }>,
        "All Mountains are Plains.",
    )],
    ..oracle::CONVERSION
};
