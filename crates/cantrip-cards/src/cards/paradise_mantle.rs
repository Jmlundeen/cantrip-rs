use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_oracle as oracle;

/// Equipped creature has "{T}: Add one mana of any color" (CR 613.1f): the Mantle's own delayed ability 0, a mana
/// ability the creature has for as long as the Mantle is attached to it. The creature is its source, so its {T} is
/// subject to summoning sickness (CR 302.6).
fn grant_mana_ability(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let o = view.state().obj(source);
    let card = o.copy.map(|c| c.card).unwrap_or(o.card);
    if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
        chars.abilities[slot] = Some(AbilityRef::delayed(card, 0));
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana(1, 0, 0, 0, 0, 0), "Equip {1}")],
    delayed: &[mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color.")],
    statics: &[StaticDef {
        text: "Equipped creature has \"{T}: Add one mana of any color.\"",
        layer: Layer::Ability,
        affects: attached_object,
        apply: grant_mana_ability,
        ..StaticDef::BLANK
    }],
    ..oracle::PARADISE_MANTLE
};
