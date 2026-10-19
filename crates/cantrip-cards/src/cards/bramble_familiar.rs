use cantrip_api::*;
use cantrip_mechanics::targets::filters;
use cantrip_mechanics::library::mill_then_put_onto_battlefield;
use cantrip_oracle as oracle;

/// "Mill seven cards. Then put a creature, enchantment, or land card from among the milled cards onto the battlefield."
/// The card is not optional: one is chosen whenever any milled card qualifies. The milled cards are found in the zone they
/// moved to (CR 701.17c), a graveyard or, by a replacement effect, exile. An Adventurer card is only its creature card in
/// those zones (CR 715.4). An Aura chooses what it enchants as it enters (CR 303.4f).
fn fetch_quest(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    mill_then_put_onto_battlefield(g, item.controller, item.object, 7, |chars| {
        chars.is(CardType::Creature) || chars.is(CardType::Enchantment) || chars.is(CardType::Land)
    })
}

/// The Adventure (CR 715): Fetch Quest.
pub static FETCH_QUEST: CardDef = CardDef {
    spell: spell(&[], fetch_quest),
    ..oracle::FETCH_QUEST
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::G), "{T}: Add {G}."),
        ability(cost("{1}{G}, {T}").and(discard(1, filters::any)), cantrip_mechanics::effects::return_this_to_hand, "{1}{G}, {T}, Discard a card: Return this creature to its owner's hand."),
    ],
    adventure: Some(&FETCH_QUEST),
    ..oracle::BRAMBLE_FAMILIAR
};

pub static EXTRA: &[&CardDef] = &[&FETCH_QUEST];
