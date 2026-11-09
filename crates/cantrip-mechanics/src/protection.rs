//! Protection (CR 702.16) printed on a permanent: "protection from [quality]". The core does what protection does - it
//! can't be the target of, enchanted or equipped by, blocked by, or dealt damage by what it is protected from (CR 702.16b-e,
//! `Chars::protected_from`); this is the static ability that gives the permanent its own protection.

use cantrip_api::*;

use crate::statics::{itself, subtype_word};

/// "Protection from [color]" (CR 702.16a, layer 6) as a static ability's `StaticDef::apply`, `COLOR` a `Color` as `u8`. The
/// color is a word of the permanent's own text (CR 612.2): a text-changing effect changes what it is protected from.
pub fn from_color<const COLOR: u8>(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    chars.protection = chars.protection.with(view.words(source).color(Color::ALL[COLOR as usize]));
}

/// The permanent's own "protection from [color]" (CR 702.16a): Kor Firewalker's "protection from red".
pub const fn from<const COLOR: u8>(text: &'static str) -> StaticDef {
    static_ability(Layer::Ability, itself, from_color::<COLOR>, text)
}

/// "Protection from [card type]" (CR 702.16a, layer 6) as a static ability's `StaticDef::apply`, `TYPE` a `CardType` as `u8`:
/// it can't be the target of, enchanted or equipped by, blocked by, or dealt damage by a source of that type (CR 702.16b-e, `Chars::protection_types`). Hexdrinker's level
/// ability applies it under a condition of its own; Emrakul, the Promised End prints it outright (`from_type`).
pub fn from_type_apply<const TYPE: u8>(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    if let Some(&t) = CardType::ALL.iter().find(|&&t| t as u8 == TYPE) {
        chars.protection_types = chars.protection_types.with(t);
    }
}

/// The permanent's own "protection from [card type]" (CR 702.16a): Emrakul, the Promised End's "protection from instants".
pub const fn from_type<const TYPE: u8>(text: &'static str) -> StaticDef {
    static_ability(Layer::Ability, itself, from_type_apply::<TYPE>, text)
}

/// "Protection from [creature type]" (CR 702.16a, layer 6) as a static ability's `StaticDef::apply`, `SUBTYPE` a `Subtype` as
/// `u16` (`lands::sub`): it can't be the target of, enchanted or equipped by, blocked by, or dealt damage by a source that is
/// that type (CR 702.16b-e, `Chars::protection_subtypes`). The type is a word of the permanent's own text (CR 612.2). Baneslayer
/// Angel (Demons, Dragons), Dragonstalker (Dragons) and Grave Bramble (Zombies) print it.
pub fn from_subtype_apply<const SUBTYPE: u16>(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    if let Some(word) = subtype_word(view, source, SUBTYPE) {
        chars.protection_subtypes = chars.protection_subtypes.with(word);
    }
}

/// The permanent's own "protection from [creature type]" (CR 702.16a): Baneslayer Angel's "protection from Demons".
pub const fn from_subtype<const SUBTYPE: u16>(text: &'static str) -> StaticDef {
    static_ability(Layer::Ability, itself, from_subtype_apply::<SUBTYPE>, text)
}

/// "Protection from each color" (CR 702.16a, layer 6) as a static ability's `StaticDef::apply`: protection from white, blue,
/// black, red and green at once, each a color word of the permanent's own text (CR 612.2). It is protection from the colors and
/// not from colorless (CR 105.2c). Iridescent Angel prints it.
pub fn from_each_color_apply(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    for color in Color::ALL {
        chars.protection = chars.protection.with(view.words(source).color(color));
    }
}

/// The permanent's own "protection from each color" (CR 702.16a): Iridescent Angel's.
pub const fn from_each_color(text: &'static str) -> StaticDef {
    static_ability(Layer::Ability, itself, from_each_color_apply, text)
}

/// "Protection from mana value N or greater" (CR 702.16a, layer 6) as a static ability's `StaticDef::apply`: it can't be the
/// target of, enchanted or equipped by, blocked by, or dealt damage by a source whose mana value (CR 202.3) is `LEAST` or more
/// (CR 702.16b-e, `Chars::protection_mana_value`). With more than one, the least rules. Mistmeadow Skulk prints it.
pub fn from_mana_value_at_least_apply<const LEAST: u32>(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.protect_from_mana_value_at_least(LEAST);
}

/// The permanent's own "protection from mana value N or greater" (CR 702.16a): Mistmeadow Skulk's "protection from mana value
/// 3 or greater".
pub const fn from_mana_value_at_least<const LEAST: u32>(text: &'static str) -> StaticDef {
    static_ability(Layer::Ability, itself, from_mana_value_at_least_apply::<LEAST>, text)
}

/// "Enchanted creature has protection from [color]. This effect doesn't remove this Aura." (CR 702.16n, layer 6) as a static
/// ability's `StaticDef::apply`, `COLOR` a `Color` as `u8`: protection in every way (CR 702.16b-e), but the Aura giving it is
/// not put into its owner's graveyard by it (`Chars::protect_sparing`). The color is a word of the Aura's text (CR 612.2).
pub fn from_color_not_removing_this_aura_apply<const COLOR: u8>(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let color = view.words(source).color(Color::ALL[COLOR as usize]);
    chars.protect_sparing(ColorSet::of(color), TypeSet::NONE, Spares::Aura(source));
}

/// An Aura's "Enchanted creature has protection from [color]. This effect doesn't remove this Aura." (CR 702.16n): Black Ward,
/// Blue Ward, Green Ward, Red Ward and White Ward print it.
pub const fn enchanted_from_color_not_removing_this_aura<const COLOR: u8>(text: &'static str) -> StaticDef {
    static_ability(Layer::Ability, crate::attach::attached_object, from_color_not_removing_this_aura_apply::<COLOR>, text)
}

/// "Enchanted creature has protection from the chosen color. This effect doesn't remove this Aura." (CR 702.16n, layer 6) as a
/// static ability's `StaticDef::apply`: the color chosen as the Aura entered (CR 607.2d), no color word, so a text-changing effect
/// leaves it (CR 612.2). An Aura with none chosen gives nothing.
pub fn from_chosen_color_not_removing_this_aura_apply(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    if let Some(color) = view.state().chosen_color(view.linked_to(source)) {
        chars.protect_sparing(ColorSet::of(color), TypeSet::NONE, Spares::Aura(source));
    }
}

/// An Aura's "Enchanted creature has protection from the chosen color. This effect doesn't remove this Aura." (CR 702.16n), its
/// color chosen as it enters (`mana::aura_enters_choosing_a_color`): Cho-Manno's Blessing, Flickering Ward, Floating Shield,
/// Pentarch Ward and Ward of Lights print it.
pub const fn enchanted_from_chosen_color_not_removing_this_aura(text: &'static str) -> StaticDef {
    static_ability(Layer::Ability, crate::attach::attached_object, from_chosen_color_not_removing_this_aura_apply, text)
}
