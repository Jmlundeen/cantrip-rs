use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

/// Haste is in the characteristics. "Selfie Shot - Whenever you cast a noncreature spell, if at least four mana was spent to cast it,
/// create a Treasure token." (CR 603.4, 111.10a): the mana spent is the total paid, whatever its kind and whatever the spell's mana
/// value; it resolves before the spell that caused it.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::SPELL_CAST, triggers::noncreature_spell_you_cast_with_at_least_mana_spent::<4>),
        tokens::create_a_treasure,
        "Selfie Shot \u{2014} Whenever you cast a noncreature spell, if at least four mana was spent to cast it, create a Treasure token.",
    )],
    ..oracle::PROMPTO_ARGENTUM
};
