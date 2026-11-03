use cantrip_api::*;
use cantrip_mechanics::{attach, effects, statics, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of your upkeep, you may pay {W}{W}. If you do, you gain 1 life." (CR 608.2, 119.3, 118.1).
fn pay_ww_to_gain_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::may_pay_to_gain_life(g, item, cost("{W}{W}"))
}

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::Ability,
            attach::attached_object,
            statics::grant_delayed_ability,
            "Enchanted land has \"At the beginning of your upkeep, you may pay {W}{W}. If you do, you gain 1 life.\"",
        ),
    ],
    delayed: &[
        triggered(
            triggers::YOUR_UPKEEP,
            pay_ww_to_gain_life,
            "At the beginning of your upkeep, you may pay {W}{W}. If you do, you gain 1 life.",
        ),
    ],
    enchant: Some(cantrip_mechanics::targets::LAND),
    ..oracle::FARMSTEAD
};
