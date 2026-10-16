use cantrip_api::*;
use cantrip_mechanics::attach::{attach_to_target, attached_object, equip};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{craft, damage_target, targets};
use cantrip_oracle as oracle;

/// "When this artifact enters, it deals 2 damage to any target."
fn two_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 2);
    Ok(())
}

/// "Equipped creature gets +2/+0."
fn plus_two_power(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 2);
}

pub static BACK: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, attach_to_target, "When this Equipment enters, attach it to target creature you control.")
            .targets(&[targets::CREATURE_YOU_CONTROL]),
        equip(mana("{2}"), "Equip {2}"),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, plus_two_power, "Equipped creature gets +2/+0.")],
    ..oracle::SOVEREIGNS_MACUAHUITL
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, two_damage, "When this artifact enters, it deals 2 damage to any target.").targets(&[targets::ANY]),
        craft::ability(mana("{2}{R}"), &craft::ARTIFACT, "Craft with artifact {2}{R}"),
    ],
    back: Some(&BACK),
    ..oracle::IDOL_OF_THE_DEEP_KING
};
