use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_mechanics::impending;
use cantrip_oracle as oracle;

/// Mill four cards, then you may return a non-Avatar creature card or a planeswalker card from your graveyard to your hand:
/// chosen from the whole graveyard as it stands after the mill, and "non-Avatar" qualifies the creature card alone.
fn mill_then_return(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.mill(you, 4);
    let avatar = g.words(item.object).subtype(Subtype::Avatar);
    let graveyard = g.state().zone(Zone::graveyard(you)).to_vec();
    let eligible: Vec<ObjectId> = graveyard
        .iter()
        .copied()
        .filter(|&c| {
            let chars = g.chars(c);
            chars.is(CardType::Planeswalker) || (chars.is(CardType::Creature) && !chars.has_subtype(avatar))
        })
        .collect();
    if let Some(chosen) = choose::cards_from(g, you, item.object, Purpose::PutIntoHand, graveyard, eligible, 1, true)?.first() {
        g.return_to_hand(&[*chosen]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    ways: &[impending::way(mana("{1}{B}"), "Impending 5\u{2014}{1}{B}")],
    enters: Some(impending::enters::<5>),
    statics: &[impending::NOT_A_CREATURE],
    abilities: &[
        triggered(cantrip_mechanics::triggers::THIS_ENTERS_OR_ATTACKS, mill_then_return, "Whenever this permanent enters or attacks, mill four cards, then you may return a non-Avatar creature card or a planeswalker card from your graveyard to your hand."),
        impending::REMOVE_A_TIME_COUNTER,
    ],
    ..oracle::OVERLORD_OF_THE_BALEMURK
};
