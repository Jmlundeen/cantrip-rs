use cantrip_api::*;
use cantrip_mechanics::{choose, choose_from_hand, targets};
use cantrip_oracle as oracle;

/// "Target player discards two cards. Put up to one land card discarded this way onto the battlefield tapped under your
/// control." The target chooses the two cards (all of a shorter hand) and they are discarded together (CR 701.9b, 101.4). Of
/// the cards discarded, in a graveyard or in exile where a replacement effect put them instead (CR 614.6, 400.7j), the spell's
/// controller chooses up to one land card
/// and puts it onto the battlefield tapped under their control (CR 608.2c, 110.2a). Nothing happens if the target is illegal
/// (CR 608.2b).
fn discard_and_steal_a_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = targets::player(g, item, 0) else { return Ok(()) };
    let chosen = choose_from_hand(g, player, item.object, 2)?;
    let discarded = g.discard_together(&chosen);
    let lands: Vec<ObjectId> =
        discarded.into_iter().filter(|&c| (g.obj(c).zone.is(ZoneKind::Graveyard) || g.obj(c).zone.is(ZoneKind::Exile)) && g.chars(c).is(CardType::Land)).collect();
    if let Some(&land) = choose::cards(g, item.controller, item.object, Purpose::PutOntoBattlefield, lands, 1, true)?.first() {
        g.put_onto_battlefield(land, item.controller, true)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PLAYER], discard_and_steal_a_land),
    ..oracle::MIND_ROOTS
};
