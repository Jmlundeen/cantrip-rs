use cantrip_api::*;
use cantrip_mechanics::storm;
use cantrip_mechanics::attach::attached_object;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Enchanted creature gets +1/+1 for each enchantment you control" (CR 613.4c): a count of the board for the Aura's
/// controller, re-read on every pass; the Aura counts itself and every token copy counts every other.
fn bonus(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let you = view.state().obj(source).controller;
    let n = view
        .state()
        .battlefield()
        .iter()
        .filter(|&&id| view.state().obj(id).controller == you && view.chars(id).is(CardType::Enchantment))
        .count() as i16;
    chars.power = chars.power.map(|v| v + n);
    chars.toughness = chars.toughness.map(|v| v + n);
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE_YOU_CONTROL),
    abilities: &[storm::trigger("Storm (When you cast this spell, copy it for each spell cast before it this turn. You may choose new targets for the copies. Copies become tokens.)")],
    statics: &[StaticDef {
        text: "Enchanted creature gets +1/+1 for each enchantment you control.",
        layer: Layer::PtModify,
        affects: attached_object,
        apply: bonus,
        ..StaticDef::BLANK
    }],
    ..oracle::TEMPEST_TECHNIQUE
};
