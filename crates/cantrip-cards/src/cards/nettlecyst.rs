use cantrip_api::*;
use cantrip_mechanics::attach::living_weapon;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;


/// "Equipped creature gets +1/+1 for each artifact and/or enchantment you control" (layer 7c): one count per
/// permanent, so one that is both counts once, read afresh on every pass.
fn plus_one_for_each(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let you = q.state().obj(source).controller;
    let count = q
        .state()
        .battlefield()
        .iter()
        .filter(|&&id| {
            q.state().obj(id).controller == you && {
                let c = q.chars(id);
                c.is(CardType::Artifact) || c.is(CardType::Enchantment)
            }
        })
        .count() as i16;
    chars.power = chars.power.map(|v| v + count);
    chars.toughness = chars.toughness.map(|v| v + count);
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, living_weapon, "Living weapon (When this Equipment enters, create a 0/0 black Phyrexian Germ creature token, then attach this to it.)"),
        equip(mana("{2}"), "Equip {2}"),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, plus_one_for_each, "Equipped creature gets +1/+1 for each artifact and/or enchantment you control.")],
    ..oracle::NETTLECYST
};


pub static EXTRA: &[&CardDef] = &[&cantrip_mechanics::tokens::GERM];
