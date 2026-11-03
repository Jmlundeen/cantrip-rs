use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Companion — Your starting deck contains only cards with mana value 3 or greater and land cards. (If this card is your chosen companion, you may put it into your hand from outside the game for {3} as a sorcery.) When Keruga enters, draw a card for each other permanent you control with mana value 3 or greater." (CR 608.2).
fn companion_starting_deck_contains_only_companion(
    q: &dyn Query,
    deck: &StartingDeck,
) -> bool {
    deck.cards
        .iter()
        .all(|&id| (i64::from(q.mana_value(id))) >= 3 || q.chars(id).is(CardType::Land))
}

/// "When Keruga enters, draw a card for each other permanent you control with mana value 3 or greater." (CR 608.2, 121.1).
fn draw_card_other_permanent_control(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let others = g
        .permanents_of(item.controller)
        .into_iter()
        .filter(|&id| id != item.object && g.mana_value(id) >= 3)
        .count();
    g.draw(item.controller, others as u32);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            draw_card_other_permanent_control,
            "When Keruga enters, draw a card for each other permanent you control with mana value 3 or greater.",
        ),
    ],
    companion: Some(companion_starting_deck_contains_only_companion),
    ..oracle::KERUGA_THE_MACROSAGE
};
