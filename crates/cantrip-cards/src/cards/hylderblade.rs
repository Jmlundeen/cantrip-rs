use cantrip_api::*;
use cantrip_mechanics::attach::{attach_to_target, attached_object, equip};
use cantrip_mechanics::triggers::YOUR_END_STEP;
use cantrip_mechanics::{history, statics, targets};
use cantrip_oracle as oracle;

/// "Equipped creature gets +3/+1" (layer 7c). Void (CR 207.2c, 603.4): at the beginning of your end step, if a nonland
/// permanent left the battlefield this turn or a spell was warped this turn, attach this Equipment to target creature you
/// control.
pub static CARD: CardDef = CardDef {
    abilities: &[
        equip(mana("{4}"), "Equip {4}"),
        triggered(
            YOUR_END_STEP,
            attach_to_target,
            "Void \u{2014} At the beginning of your end step, if a nonland permanent left the battlefield this turn or a spell was warped this turn, attach this Equipment to target creature you control.",
        )
        .targets(&[targets::CREATURE_YOU_CONTROL])
        .condition(|q, _| history::void(q)),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, statics::plus_pt::<3, 1>, "Equipped creature gets +3/+1.")],
    ..oracle::HYLDERBLADE
};
