use cantrip_api::*;
use cantrip_mechanics::craft;
use cantrip_oracle as oracle;

/// "Craft with one or more": one or more other permanents you control and/or cards from your graveyard, of any kind (CR
/// 702.167a); the source is never among the candidates.
fn anything(_: &dyn Query, _: &Asker, _: ObjectId) -> bool {
    true
}

static ONE_OR_MORE: Materials = craft::materials(1, true, anything, "");

/// The colors among the exiled cards used to craft `effigy` (CR 105.2, 702.167c), each card's colors as it is in exile.
fn crafted_colors(s: &GameState, chars_of: impl Fn(ObjectId) -> Chars, effigy: ObjectId) -> ColorSet {
    craft::used_to_craft(s, effigy).into_iter().fold(ColorSet::NONE, |all, card| ColorSet(all.0 | chars_of(card).colors.0))
}

/// "Sunbird Effigy's power and toughness are each equal to the number of colors among the exiled cards used to craft it"
/// (CR 604.3): none crafted with, or none of them colored, a 0/0.
fn colors_size(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let n = crafted_colors(q.state(), |c| q.chars(c), source).iter().count() as i16;
    chars.power = Some(n);
    chars.toughness = Some(n);
}

/// "{T}: For each color among the exiled cards used to craft this creature, add one mana of that color." (CR 106.5: none
/// when there is none.)
fn add_crafted_colors(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    for color in crafted_colors(g.state(), |c| g.chars(c), asker.source).iter() {
        g.add_mana(asker.controller, color.mana(), 1);
    }
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_by(add_crafted_colors), "{T}: For each color among the exiled cards used to craft this creature, add one mana of that color.")],
    statics: &[characteristic_defining(Layer::PtDefine, colors_size, "Sunbird Effigy's power and toughness are each equal to the number of colors among the exiled cards used to craft it.").lost_pt(0, 0)],
    ..oracle::SUNBIRD_EFFIGY
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color."),
        craft::ability(mana("{5}"), &ONE_OR_MORE, "Craft with one or more {5}"),
    ],
    back: Some(&BACK),
    ..oracle::SUNBIRD_STANDARD
};
