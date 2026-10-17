use cantrip_api::*;
use cantrip_mechanics::statics::{self, every_basic_land_type, your_lands};
use cantrip_oracle as oracle;

/// "Each nonland permanent you control": by control, judged on what layers 1 to 4 made it (CR 613.1e), Leyline itself
/// among them.
fn your_nonland_permanents(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let s = q.state();
    !so_far.is(CardType::Land) && s.obj(affected).controller == s.obj(source).controller
}

/// "... is all colors" (CR 105.2, 613.1e, layer 5): white, blue, black, red and green, whatever it was. "All colors" names
/// no color word a text-changing effect could change (CR 612.2).
fn all_colors(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.colors = ColorSet(0b11111);
}

pub static CARD: CardDef = CardDef {
    // "If this card is in your opening hand, you may begin the game with it on the battlefield." (CR 103.6a)
    opening_hand: Some(statics::BEGIN_ON_THE_BATTLEFIELD),
    statics: &[
        static_ability(Layer::Color, your_nonland_permanents, all_colors, "Each nonland permanent you control is all colors."),
        static_ability(
            Layer::Type,
            your_lands,
            every_basic_land_type,
            "Lands you control are every basic land type in addition to their other types.",
        ),
    ],
    ..oracle::LEYLINE_OF_THE_GUILDPACT
};
