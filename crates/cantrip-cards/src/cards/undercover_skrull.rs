use cantrip_api::*;
use cantrip_mechanics::statics::{itself_with_two_creature_cards_in_your_graveyard, plus_pt};
use cantrip_oracle as oracle;

/// "... is all creature types" (CR 205.3m, 613.1d, layer 4): every creature type in addition to the types it has. "All creature
/// types" names no creature type, so no text-changing effect changes it (CR 612.2).
fn all_creature_types(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    if chars.is(CardType::Creature) {
        chars.subtypes = chars.subtypes.union(SubtypeSet::CREATURE_TYPES);
    }
}

/// "As long as there are two or more creature cards in your graveyard, this creature gets +2/+2 and is all creature types."
/// "{T}: Add one mana of any color." (CR 106.1b, 605.1a)
pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color.")],
    statics: &[
        static_ability(Layer::PtModify, itself_with_two_creature_cards_in_your_graveyard, plus_pt::<2, 2>, "As long as there are two or more creature cards in your graveyard, this creature gets +2/+2."),
        static_ability(Layer::Type, itself_with_two_creature_cards_in_your_graveyard, all_creature_types, "As long as there are two or more creature cards in your graveyard, this creature is all creature types."),
    ],
    ..oracle::UNDERCOVER_SKRULL
};
