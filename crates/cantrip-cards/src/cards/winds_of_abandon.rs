use cantrip_api::*;
use cantrip_mechanics::{overload, search_library};
use cantrip_oracle as oracle;

/// "Creature you don't control" (CR 109.5): another player's, including a teammate's.
fn creature_you_dont_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if is_live(q, id) && q.is_on_battlefield(id)
        && q.chars(id).is(CardType::Creature) && q.controller(id) != asker.controller)
}

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
        let found = search_library(g, player, item.object, count, true, |q, c| {
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
    spell: Some(SpellDef { targets: &[TargetSpec { min: 1, max: 1, filter: creature_you_dont_control }], effect }),
    overload: Some(mana(4, 2, 0, 0, 0, 0)),
    ..oracle::WINDS_OF_ABANDON
};
