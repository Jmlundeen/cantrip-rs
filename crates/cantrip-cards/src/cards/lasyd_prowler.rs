use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, lands_you_control, targets};
use cantrip_oracle as oracle;

/// "When this creature enters, you may mill cards equal to the number of lands you control" (CR 701.17a, 701.17b): the lands as
/// the ability resolves; a player can't choose to mill more cards than their library holds, so then nothing is asked.
fn may_mill_a_card_per_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let count = lands_you_control(g, item.controller);
    effects::may_mill(g, item, count)
}

/// "Put X +1/+1 counters on target creature, where X is the number of land cards in your graveyard" (CR 122.1b, 107.3): counted
/// as the ability resolves, after this card has been exiled as a cost, and nothing if the target is no longer legal (CR 608.2b).
fn renew(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let graveyard = g.state().zone(Zone::graveyard(item.controller)).to_vec();
    let lands = graveyard.into_iter().filter(|&card| g.chars(card).is(CardType::Land)).count();
    g.add_counters(creature, CounterKind::PlusOne, lands as i32);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    // Renew is an ability word (CR 207.2c): the ability is an ordinary activated ability of the card in the graveyard.
    abilities: &[
        triggered(THIS_ENTERS, may_mill_a_card_per_land, "When this creature enters, you may mill cards equal to the number of lands you control."),
        ability(cost("{1}{G}").and(EXILE_THIS), renew, "Renew \u{2014} {1}{G}, Exile this card from your graveyard: Put X +1/+1 counters on target creature, where X is the number of land cards in your graveyard. Activate only as a sorcery.")
            .from(ZoneKind::Graveyard)
            .sorcery_speed()
            .targets(&[targets::CREATURE]),
    ],
    ..oracle::LASYD_PROWLER
};
