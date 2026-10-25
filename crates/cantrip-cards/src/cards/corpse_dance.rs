use cantrip_api::*;
use cantrip_mechanics::haste_until_end_of_turn;
use cantrip_mechanics::replacements::BUYBACK;
use cantrip_oracle as oracle;

/// Return the top creature card of your graveyard to the battlefield. It gains haste until end of turn. Exile it at
/// the beginning of the next end step. Nothing is chosen or targeted: a graveyard is ordered (CR 404.2), its last
/// card its top one, and cards put into it together are in the order their owner chose (CR 404.3).
fn dance(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
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
    cantrip_mechanics::delayed::at_next_end_step(g, item, 0, Target::Object(returned));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], dance),
    kicker: kicker(mana("{2}")),
    replacements: &[BUYBACK],
    delayed: &[delayed_trigger(cantrip_mechanics::delayed::exile_subject, "Exile it.")],
    // "The top creature card of your graveyard" (CR 404.2): its owner orders cards put there together (CR 404.3).
    reads_graveyard_order: true,
    ..oracle::CORPSE_DANCE
};
