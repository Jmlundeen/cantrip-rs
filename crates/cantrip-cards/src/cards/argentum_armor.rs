use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_oracle as oracle;

fn destroy_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

fn plus_six(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 6);
    chars.toughness = chars.toughness.map(|v| v + 6);
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever equipped creature attacks, destroy target permanent.",
            listens: EventMask::ATTACKS,
            matches: cantrip_mechanics::triggers::equipped_creature_attacks,
            targets: &[targets::PERMANENT],
            effect: destroy_it,
            ..TriggeredDef::BLANK
        }),
        equip(mana(6, 0, 0, 0, 0, 0), "Equip {6}"),
    ],
    statics: &[StaticDef {
        text: "Equipped creature gets +6/+6.",
        layer: Layer::PtModify,
        affects: attached_object,
        apply: plus_six,
        ..StaticDef::BLANK
    }],
    ..oracle::ARGENTUM_ARMOR
};
