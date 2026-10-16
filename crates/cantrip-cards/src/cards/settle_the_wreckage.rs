use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::library::{Find, search_to_battlefield};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Exile all attacking creatures target player controls. That player may search their library for that many basic land
/// cards, put those cards onto the battlefield tapped, then shuffle." The spell targets only the player (CR 115.1), so
/// hexproof creatures are exiled too. The creatures go in one move (CR 608.2f); "that many" is the number of attacking
/// creatures exiled by the spell, those a replacement sent elsewhere included (CR 608.2c). A spell whose only target
/// is illegal is not resolved at all (CR 608.2b). The "may" is asked of that player; declining skips search and shuffle.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = item.targets[0].player() else { return Ok(()) };
    let attackers: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.controller(id) == player && g.is_attacking(id) && g.chars(id).is(CardType::Creature))
        .collect();
    let count = g.exile_all(&attackers).len().min(u8::MAX as usize) as u8;
    // "That many" is zero: there is nothing to search for, so no search is offered (and shuffling an unsearched library
    // changes nothing anyone can see).
    if count == 0 || !effects::ask_may(g, player, item.object, Purpose::Optional)? {
        return Ok(());
    }
    search_to_battlefield(g, player, item.object, Find::ONE.up_to(count), true, |q, c| {
        let chars = q.chars(c);
        chars.is(CardType::Land) && chars.supertypes.basic
    })?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PLAYER], effect),
    ..oracle::SETTLE_THE_WRECKAGE
};
