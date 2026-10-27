use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::{effects, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When this Equipment enters, target creature an opponent controls gets -2/-2 until end of turn." (CR 603.6a,
/// 603.3d, 601.2c, 611.2a, 514.2): the target is chosen as the ability is put on the stack, and the temporary modifier ends
/// during cleanup.
pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            effects::target_creature_gets::<-2, -2>,
            "When this Equipment enters, target creature an opponent controls gets -2/-2 until end of turn.",
        )
        .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS]),
        equip(mana("{2}"), "Equip {2}"),
    ],
    // "Equipped creature gets +2/+0." (CR 301.5a, 611.3a–b): this continuous effect applies only to the creature
    // currently attached to the Bat'leth.
    statics: &[static_ability(
        Layer::PtModify,
        attached_object,
        cantrip_mechanics::statics::plus_pt::<2, 0>,
        "Equipped creature gets +2/+0.",
    )],
    ..oracle::BATLETH
};
