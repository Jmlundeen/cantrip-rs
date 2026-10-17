use cantrip_api::*;
use cantrip_mechanics::hand;
use cantrip_oracle as oracle;

/// "You may put a creature card from your hand onto the battlefield. That creature gains haste. Sacrifice that creature
/// at the beginning of the next end step." Up to one creature card; nothing is asked when the hand has none. The haste
/// has no end but the object (CR 611.2a), and the delayed sacrifice names the object (CR 603.7c), so a creature that
/// has since become a new object (CR 400.7) is left alone. When it is spliced onto an Arcane spell (CR 702.47) `item`
/// is that spell, so the delayed ability is this card's own, not the item's.
fn put_creature_in(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let Some(returned) = hand::may_put_onto_battlefield(g, player, item.object, |c| c.is(CardType::Creature))? else { return Ok(()) };
    if !is_live(g, returned) || !g.is_on_battlefield(returned) {
        return Ok(());
    }
    cantrip_mechanics::effects::gain_haste(g, item.object, returned);
    let this = g.card_id(&CARD);
    g.delayed_trigger(
        DelayedWhen::Step { step: Step::End, on_turn_of: None },
        item.object,
        player,
        AbilityRef::delayed(this, 0),
        Target::Object(returned),
    );
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], put_creature_in),
    splice: Some(mana("{2}{R}{R}")),
    delayed: &[delayed_trigger(cantrip_mechanics::delayed::sacrifice_subject, "Sacrifice that creature at the beginning of the next end step.")],
    ..oracle::THROUGH_THE_BREACH
};
