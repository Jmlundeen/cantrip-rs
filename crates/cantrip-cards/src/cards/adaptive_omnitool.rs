use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::library::{look_at_top, Rest};
use cantrip_oracle as oracle;

/// Equipped creature gets +1/+1 for each artifact you control (layer 7c): the Omnitool itself among them.
fn artifact_bonus(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let you = q.state().obj(source).controller;
    let state = q.state();
    let count = state.battlefield().iter().filter(|&&id| state.obj(id).controller == you && q.chars(id).is(CardType::Artifact)).count();
    let bonus = count as i16;
    chars.power = chars.power.map(|v| v + bonus);
    chars.toughness = chars.toughness.map(|v| v + bonus);
}

/// Look at the top six cards of your library. You may reveal an artifact card from among them and put it into your
/// hand. Put the rest on the bottom of your library in a random order. With no artifact card among them nobody is
/// asked.
fn look(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    look_at_top(g, item.controller, item.object, 6, (0, 1), true, |c| c.is(CardType::Artifact), Rest::BottomRandom)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(cantrip_mechanics::triggers::EQUIPPED_CREATURE_ATTACKS, look, "Whenever equipped creature attacks, look at the top six cards of your library. You may reveal an artifact card from among them and put it into your hand. Put the rest on the bottom of your library in a random order."),
        equip(mana(3, 0, 0, 0, 0, 0), "Equip {3}"),
    ],
    statics: &[StaticDef {
        text: "Equipped creature gets +1/+1 for each artifact you control.",
        layer: Layer::PtModify,
        affects: attached_object,
        apply: artifact_bonus,
        ..StaticDef::BLANK
    }],
    ..oracle::ADAPTIVE_OMNITOOL
};
