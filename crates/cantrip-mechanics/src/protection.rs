//! Protection (CR 702.16) printed on a permanent: "protection from [quality]". The core does what protection does - it
//! can't be the target of, enchanted or equipped by, blocked by, or dealt damage by what it is protected from (CR 702.16b-e,
//! `Chars::protected_from`); this is the static ability that gives the permanent its own protection.

use cantrip_api::*;

use crate::statics::itself;

/// "Protection from [color]" (CR 702.16a, layer 6) as a static ability's `StaticDef::apply`, `COLOR` a `Color` as `u8`. The
/// color is a word of the permanent's own text (CR 612.2): a text-changing effect changes what it is protected from.
pub fn from_color<const COLOR: u8>(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    chars.protection = chars.protection.with(view.words(source).color(Color::ALL[COLOR as usize]));
}

/// The permanent's own "protection from [color]" (CR 702.16a): Kor Firewalker's "protection from red".
pub const fn from<const COLOR: u8>(text: &'static str) -> StaticDef {
    static_ability(Layer::Ability, itself, from_color::<COLOR>, text)
}
