use cantrip_api::*;
use cantrip_mechanics::statics::your_creatures;
use cantrip_oracle as oracle;

/// "... have flying, first strike, vigilance, trample, haste, and protection from black and from red" (CR 702.9, 702.7,
/// 702.20, 702.19, 702.10, 702.16, layer 6). The colors are read through the words of the Memorial's text (CR 612).
fn grant(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    chars.keywords = chars
        .keywords
        .with(Keyword::Flying)
        .with(Keyword::FirstStrike)
        .with(Keyword::Vigilance)
        .with(Keyword::Trample)
        .with(Keyword::Haste);
    let words = view.words(source);
    chars.protection = chars.protection.with(words.color(Color::Black)).with(words.color(Color::Red));
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::Ability,
        your_creatures,
        grant,
        "Creatures you control have flying, first strike, vigilance, trample, haste, and protection from black and from red.",
    )],
    ..oracle::AKROMAS_MEMORIAL
};
