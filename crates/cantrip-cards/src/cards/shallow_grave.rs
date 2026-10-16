use cantrip_api::*;
use cantrip_mechanics::{delayed, haste_until_end_of_turn};
use cantrip_oracle as oracle;

/// Return the top creature card of your graveyard to the battlefield. It gains haste until end of turn. Exile it at
/// the beginning of the next end step. Nothing is chosen or targeted: a graveyard is ordered (CR 404.3) and its last
/// card is its top one.
fn grave(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let top_creature = g
        .state()
        .zone(Zone::graveyard(player))
        .iter()
        .rev()
        .copied()
        .find(|&id| g.chars(id).is(CardType::Creature));
    let Some(card) = top_creature else { return Ok(()) };
    let returned = g.put_onto_battlefield(card, player, false)?;
    if !g.is_on_battlefield(returned) {
        return Ok(());
    }
    haste_until_end_of_turn(g, item.object, returned);
    delayed::at_next_end_step(g, item, 0, Target::Object(returned));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], grave),
    delayed: &[delayed_trigger(delayed::exile_subject, "Exile it.")],
    ..oracle::SHALLOW_GRAVE
};
