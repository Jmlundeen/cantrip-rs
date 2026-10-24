use cantrip_api::*;
use cantrip_mechanics::{lands, overload, targets};
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

    let mut searchers: Vec<(PlayerId, u32)> = Vec::new();
    for &player in &controllers {
        match searchers.iter_mut().find(|(p, _)| *p == player) {
            Some((_, n)) => *n += 1,
            None => searchers.push((player, 1)),
        }
    }
    lands::each_searches_basic_lands_together(g, &searchers, item.object, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_DONT_CONTROL], effect),
    overload: Some(mana("{4}{W}{W}")),
    ..oracle::WINDS_OF_ABANDON
};
