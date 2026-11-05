use cantrip_api::*;
use cantrip_mechanics::library::{mill_then_return_a_card, object_has_name, search_zone_and_or_library_onto_battlefield};
use cantrip_mechanics::select::{self, up_to, Selection};
use cantrip_oracle as oracle;

const SEARCH: Question = Question::new(
    "SearchLibrary",
    "search your graveyard, hand, and library for a card named Altanak, the Thrice-Called?",
    "Altanak, found in any of them, goes onto the battlefield, and you shuffle. If you don't, you may still take it from your graveyard or hand",
    "Search all three",
    "Graveyard and hand only",
);

/// "Mill three cards. Then you may return a creature or land card from your graveyard to your hand." (CR 701.17, 608.2c): the
/// card is chosen from the whole graveyard as the mill left it, a card just milled included, and the player may choose none.
fn spell_effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    mill_then_return_a_card(g, item, 3, select::CREATURE_OR_LAND_CARD)
}

/// "Two other cards named Say Its Name" (CR 201.2): the fixed name, not whatever a text-changing or copy effect
/// would make the card's current one.
fn card_named_say_its_name(q: &dyn Query, _: &Asker, card: ObjectId) -> bool {
    object_has_name(q, card, q.card_id(&CARD))
}

/// "Search your graveyard, hand, and/or library for a card named Altanak, the Thrice-Called and put it onto the battlefield.
/// If you search your library this way, shuffle." (CR 701.23, 701.24): the cost has already exiled this card and two others.
/// The player chooses whether the library is searched too; if so the one card (or none, in a hidden zone, CR 701.23b) is
/// chosen from all three zones at once, and the library is shuffled afterwards, found or not.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let altanak = g.card_id(&super::altanak_the_thrice_called::CARD);
    let rule = Selection::new(up_to(1)).with(select::named(altanak));
    search_zone_and_or_library_onto_battlefield(g, you, item.object, &[Zone::graveyard(you), Zone::hand(you)], &SEARCH, rule)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], spell_effect),
    abilities: &[ability(
        EXILE_THIS.and(exile_from_graveyard_of(2, card_named_say_its_name)),
        search,
        "Exile this card and two other cards named Say Its Name from your graveyard: Search your graveyard, hand, and/or library for a card named Altanak, the Thrice-Called and put it onto the battlefield. If you search your library this way, shuffle. Activate only as a sorcery.",
    )
    .from(ZoneKind::Graveyard)
    .sorcery_speed()],
    ..oracle::SAY_ITS_NAME
};
