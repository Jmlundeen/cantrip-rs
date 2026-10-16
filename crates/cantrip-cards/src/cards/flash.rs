use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// You may put a creature card from your hand onto the battlefield. If you do, sacrifice it unless you pay its mana
/// cost reduced by {2}. The card is chosen as the spell resolves (CR 608.2d); one that cannot be paid for (a card with
/// no mana cost) is sacrificed outright.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let hand = g.state().zone(Zone::hand(player)).to_vec();
    let eligible: Vec<ObjectId> = hand.iter().copied().filter(|&c| g.chars(c).is(CardType::Creature)).collect();
    let chosen = choose::cards_from(g, player, item.object, Purpose::PutOntoBattlefield, hand, eligible, 1, true)?;
    let Some(&card) = chosen.first() else { return Ok(()) };
    let def = g.card(g.obj(card).card);
    let entered = g.put_onto_battlefield(card, player, false)?;
    if !targets::permanent(g, entered) {
        return Ok(());
    }
    let mut cost = def.cost;
    cost.generic = cost.generic.saturating_sub(2);
    if !def.has_cost || !g.unless_pays(player, item.object, entered, cost)? {
        g.sacrifice(&[entered]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::FLASH };
