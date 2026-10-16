use cantrip_api::*;
use cantrip_mechanics::{overload, search_library, targets};
use cantrip_oracle as oracle;

/// Exile target creature you don't control. For each creature exiled this way, its controller searches their library
/// for a basic land card. Those players put those cards onto the battlefield tapped, then shuffle. Overloaded
/// (CR 702.96a), each creature you don't control is exiled, together. Each former controller searches once, for as
/// many lands as they lost creatures (CR 701.23h), in turn order; the lands enter together before anyone shuffles.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures =
        overload::affected(g, item, |chars, controller| chars.is(CardType::Creature) && controller != item.controller);
    let controllers: Vec<PlayerId> = creatures.iter().map(|&c| g.controller(c)).collect();
    g.exile_all(&creatures);

    let order: Vec<PlayerId> = g.state().apnap().filter(|p| controllers.contains(p)).collect();
    let mut lands: Vec<(ObjectId, PlayerId, bool)> = Vec::new();
    for &player in &order {
        let count = controllers.iter().filter(|&&p| p == player).count() as u8;
        let found = search_library(g, player, player, item.object, SearchDestination::BattlefieldTapped, count, true, |q, c| {
            let chars = q.chars(c);
            chars.is(CardType::Land) && chars.supertypes.basic
        })?;
        lands.extend(found.into_iter().map(|land| (land, player, true)));
    }
    g.put_all_onto_battlefield_as(&lands)?;
    for &player in &order {
        g.shuffle(player);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_DONT_CONTROL], effect),
    overload: Some(mana("{4}{W}{W}")),
    ..oracle::WINDS_OF_ABANDON
};
