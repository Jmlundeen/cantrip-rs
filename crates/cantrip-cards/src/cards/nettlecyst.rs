use cantrip_api::*;
use cantrip_mechanics::attach::living_weapon;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::triggers::this_enters;
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
        AbilityDef::Triggered(TriggeredDef {
            text: "Living weapon (When this Equipment enters, create a 0/0 black Phyrexian Germ creature token, then attach this to it.)",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: living_weapon,
            ..TriggeredDef::BLANK
        }),
        equip(mana(2, 0, 0, 0, 0, 0), "Equip {2}"),
    ],
    statics: &[StaticDef {
        text: "Equipped creature gets +1/+1 for each artifact and/or enchantment you control.",
        layer: Layer::PtModify,
        affects: attached_object,
        apply: plus_one_for_each,
        ..StaticDef::BLANK
    }],
    ..oracle::NETTLECYST
};


pub static EXTRA: &[&CardDef] = &[&cantrip_mechanics::tokens::GERM];
