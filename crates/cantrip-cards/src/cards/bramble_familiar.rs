use cantrip_api::*;
use cantrip_mechanics::targets::filters;
use cantrip_mechanics::choose;
use cantrip_oracle as oracle;

/// "Return this creature to its owner's hand." Nothing if it has left the battlefield by the time the ability resolves:
/// the verb returns no object that is no longer the one it was (CR 400.7).
fn return_this(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.return_to_hand(&[item.object]);
    Ok(())
}

/// "Mill seven cards. Then put a creature, enchantment, or land card from among the milled cards onto the battlefield."
/// The card is not optional: one is chosen whenever any milled card qualifies. The milled cards are found in the zone they
/// moved to (CR 701.17c), a graveyard or, by a replacement effect, exile. An Adventurer card is only its creature card in
/// those zones (CR 715.4). An Aura chooses what it enchants as it enters (CR 303.4f).
fn fetch_quest(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let among = g.mill(player, 7);
    let eligible: Vec<ObjectId> = among
        .iter()
        .copied()
        .filter(|&c| {
            let chars = g.chars(c);
            chars.is(CardType::Creature) || chars.is(CardType::Enchantment) || chars.is(CardType::Land)
        })
        .collect();
    if let Some(card) = choose::card_from(g, player, item.object, Purpose::PutOntoBattlefield, among, eligible)? {
        g.put_onto_battlefield(card, player, false)?;
    }
    Ok(())
}

/// The Adventure (CR 715): Fetch Quest.
pub static FETCH_QUEST: CardDef = CardDef {
    spell: spell(&[], fetch_quest),
    ..oracle::FETCH_QUEST
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::G), "{T}: Add {G}."),
        ability(cost("{1}{G}, {T}").and(discard(1, filters::any)), return_this, "{1}{G}, {T}, Discard a card: Return this creature to its owner's hand."),
    ],
    adventure: Some(&FETCH_QUEST),
    ..oracle::BRAMBLE_FAMILIAR
};

pub static EXTRA: &[&CardDef] = &[&FETCH_QUEST];
