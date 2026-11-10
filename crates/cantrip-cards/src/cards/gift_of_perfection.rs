use cantrip_api::*;
use cantrip_mechanics::{assimilate, choose, library, select};
use cantrip_oracle as oracle;

/// "Each opponent reveals cards from the top of their library until they reveal a creature card. Assimilate one of the
/// creature cards revealed this way. Then each opponent puts the other cards they revealed this way on the bottom of
/// their library in a random order." Each opponent reveals their own cards (CR 701.20a), one opponent at a time in turn
/// order (CR 101.4); the controller then chooses one of the creature cards revealed from among all of them and assimilates
/// it (onto the battlefield under their control as a Borg artifact creature with a +1/+1 counter, CR 205.1b, 614.12);
/// every other revealed card goes to the bottom of its owner's library in a random order (CR 401.4). A player whose
/// library holds no creature card reveals all of it and contributes nothing to assimilate.
fn gift_of_perfection(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let opponents: Vec<PlayerId> = g.state().seats().filter(|&p| p != you && g.state().in_game(p)).collect();
    let mut revealed: Vec<(PlayerId, Vec<ObjectId>)> = Vec::new();
    let mut creature_cards: Vec<ObjectId> = Vec::new();
    for opponent in opponents {
        let (found, rest) = library::reveal_until(g, opponent, &|q, card| q.chars(card).is(CardType::Creature));
        let mut theirs = rest;
        if let Some(card) = found {
            creature_cards.push(card);
            theirs.insert(0, card);
        }
        revealed.push((opponent, theirs));
    }
    let mut taken = None;
    if !creature_cards.is_empty() {
        let chosen = choose::select(
            g,
            you,
            item.object,
            Purpose::PutOntoBattlefield,
            creature_cards.clone(),
            select::creature_cards(select::exactly(1)),
        )?;
        // "Assimilate one" is mandatory, so the choice always gives one card (CR 608.2d).
        let card = chosen.first().copied().unwrap_or(creature_cards[0]);
        assimilate::assimilate(g, item, card)?;
        taken = Some(card);
    }
    for (owner, cards) in revealed {
        let bottom: Vec<ObjectId> = cards.into_iter().filter(|&c| Some(c) != taken).collect();
        library::rest_to_bottom_in_random_order(g, owner, &bottom);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], gift_of_perfection), ..oracle::GIFT_OF_PERFECTION };
