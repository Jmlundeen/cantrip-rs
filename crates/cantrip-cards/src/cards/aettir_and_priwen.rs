use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_oracle as oracle;

/// "Equipped creature has base power and toughness X/X, where X is your life total" (CR 613.4b, layer 7b): `source`'s controller's
/// life total as the layers compute it, so it follows the life total as it changes (the ruling). A life total below zero makes
/// X negative, as the rules have it.
fn base_pt_equal_to_your_life_total(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let state = q.state();
    let x = state.player(state.obj(source).controller).life;
    chars.power = Some(x);
    chars.toughness = Some(x);
}

pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana("{5}"), "Equip {5}")],
    statics: &[static_ability(
        Layer::PtSet,
        attached_object,
        base_pt_equal_to_your_life_total,
        "Equipped creature has base power and toughness X/X, where X is your life total.",
    )],
    ..oracle::AETTIR_AND_PRIWEN
};
