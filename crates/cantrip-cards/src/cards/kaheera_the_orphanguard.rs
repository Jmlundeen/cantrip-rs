use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

const TEXT: &str = "Each other creature you control that's a Cat, Elemental, Nightmare, Dinosaur, or Beast gets +1/+1 and has vigilance.";

/// The five creature types Kaheera's text names, in its order.
const TYPES: [Subtype; 5] = [Subtype::Cat, Subtype::Elemental, Subtype::Nightmare, Subtype::Dinosaur, Subtype::Beast];

/// "Companion - Each creature card in your starting deck is a Cat, Elemental, Nightmare, Dinosaur, or Beast card" (CR 702.139a):
/// a card with any of the five types qualifies; a noncreature card, whatever its subtypes, is not asked. A card outside the
/// game has no text-changing effect on it, so the types are the printed ones.
fn creature_cards_are_its_types(q: &dyn Query, deck: &StartingDeck) -> bool {
    deck.cards.iter().all(|&card| {
        let c = q.chars(card);
        !c.is(CardType::Creature) || TYPES.iter().any(|&t| c.has_subtype(t))
    })
}

/// "Each other creature you control that's a Cat, Elemental, Nightmare, Dinosaur, or Beast" (CR 613.1f): another creature of
/// Kaheera's controller as the earlier layers made it, with one of the five types, each read through the words of her text
/// (CR 612.2). One creature of several of the types is still one object, so it is affected once (the rulings).
fn other_of_her_types(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    statics::other_creatures_you_control(q, source, affected, so_far)
        && TYPES.iter().any(|&t| so_far.has_subtype(q.words(source).subtype(t)))
}

pub static CARD: CardDef = CardDef {
    companion: Some(creature_cards_are_its_types),
    statics: &[
        static_ability(Layer::Ability, other_of_her_types, statics::have_vigilance, TEXT),
        static_ability(Layer::PtModify, other_of_her_types, statics::plus_pt::<1, 1>, TEXT).continues(),
    ],
    ..oracle::KAHEERA_THE_ORPHANGUARD
};
