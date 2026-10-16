use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Sphinx of the Guildpact is all colors": a characteristic-defining ability (CR 604.3, 105.2), so it works in every
/// zone (the ruling), in layer 5.
fn all_colors(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.colors = ColorSet(0x1f);
}

pub static CARD: CardDef = CardDef {
    // "Hexproof from monocolored" (CR 702.11d): a colorless spell or source is not monocolored (the ruling).
    hexproof_from: HexproofFrom::MONOCOLORED,
    statics: &[characteristic_defining(Layer::Color, all_colors, "Sphinx of the Guildpact is all colors.")],
    ..oracle::SPHINX_OF_THE_GUILDPACT
};
