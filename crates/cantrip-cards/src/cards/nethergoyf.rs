use cantrip_api::*;
use cantrip_mechanics::escape;
use cantrip_oracle as oracle;

/// "Its power is equal to the number of card types among cards in your graveyard and its toughness is equal to that number
/// plus 1" (CR 604.3, 205.2a): counted over its controller's graveyard (the engine keeps the owner as controller off the battlefield and stack, CR 108.4a;
/// a spell has its caster as controller, CR 108.4).
fn size(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let state = q.state();
    let obj = state.obj(source);
    let mut seen = TypeSet::NONE;
    for &card in state.zone(Zone::graveyard(obj.controller)) {
        for t in q.chars(card).types.iter() {
            seen = seen.with(t);
        }
    }
    let types = seen.iter().count() as i16;
    chars.power = Some(types);
    chars.toughness = Some(types + 1);
}

pub static CARD: CardDef = CardDef {
    statics: &[characteristic_defining(Layer::PtDefine, size, "This creature's power is equal to the number of card types among cards in your graveyard and its toughness is equal to that number plus 1.").lost_pt(0, 1)],
    ways: &[escape::way_by_types(
        mana("{2}{B}"),
        4,
        "Escape\u{2014}{2}{B}, Exile any number of other cards from your graveyard with four or more card types among them.",
    )],
    ..oracle::NETHERGOYF
};
