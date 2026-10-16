use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, mana, targets};
use cantrip_oracle as oracle;

/// "+1/+1 for each creature and/or enchantment you control" (layer 7c): "you" is the Aura's controller (CR 109.5),
/// wherever the enchanted creature is, and a permanent that is both counts once. The Aura counts itself.
fn apply(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let state = view.state();
    let you = state.obj(source).controller;
    let n = state
        .battlefield()
        .iter()
        .filter(|&&id| {
            let c = view.chars(id);
            state.obj(id).controller == you && (c.is(CardType::Creature) || c.is(CardType::Enchantment))
        })
        .count() as i16;
    chars.power = chars.power.map(|v| v + n);
    chars.toughness = chars.toughness.map(|v| v + n);
}

pub static BACK: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_one_of(&[Color::Green, Color::White]), "{T}: Add {G} or {W}.")],
    enters: Some(mana::enters_tapped),
    ..oracle::HAVEN_OF_THE_HARVEST
};

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    statics: &[StaticDef {
        text: "Enchanted creature gets +1/+1 for each creature and/or enchantment you control.",
        layer: Layer::PtModify,
        affects: attached_object,
        apply,
        ..StaticDef::BLANK
    }],
    back: Some(&BACK),
    ..oracle::STRENGTH_OF_THE_HARVEST
};
