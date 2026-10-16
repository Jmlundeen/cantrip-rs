use cantrip_api::*;
use cantrip_mechanics::{attach::{attached_object, equip}, statics::have_haste, triggers::ward};
use cantrip_oracle as oracle;

fn plus_one_power(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 1);
}

/// The equipped creature has ward {1} (CR 702.21a): the Boots' own delayed ability 0, a triggered ability that the
/// creature controls and that is gone the moment the Boots stop being attached to it.
fn grant_ward(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let o = view.state().obj(source);
    let card = o.copy.map(|c| c.card).unwrap_or(o.card);
    if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
        chars.abilities[slot] = Some(AbilityRef::delayed(card, 0));
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana("{1}"), "Equip {1}")],
    delayed: &[ward::<1>("Ward {1} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {1}.)")],
    statics: &[
        static_ability(Layer::PtModify, attached_object, plus_one_power, "Equipped creature gets +1/+0."),
        static_ability(Layer::Ability, attached_object, have_haste, "Equipped creature has haste."),
        static_ability(Layer::Ability, attached_object, grant_ward, "Equipped creature has ward {1}."),
    ],
    ..oracle::LAVASPUR_BOOTS
};
