use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "Protection from black" (CR 702.16a): printed, so its color word is a word of its text (CR 612.2).
    protection: ColorSet::of(Color::Black),
    ..oracle::WHITE_KNIGHT
};
