use cantrip_api::*;
use cantrip_mechanics::hand;
use cantrip_oracle as oracle;

/// You may put a creature card from your hand onto the battlefield. It gains haste (no duration is stated, so for as long as
/// it is that object, CR 611.2a). Sacrifice it at the beginning of
/// the next end step. Nobody is asked with no creature card in hand.
fn put_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(returned) = hand::may_put_onto_battlefield(g, item.controller, item.object, |c| c.is(CardType::Creature))? else { return Ok(()) };
    if !g.is_on_battlefield(returned) {
        return Ok(());
    }
    cantrip_mechanics::effects::gain_haste(g, item.object, returned);
    cantrip_mechanics::delayed::at_next_end_step(g, item, 0, Target::Object(returned));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{R}"), put_creature, "{R}: You may put a creature card from your hand onto the battlefield. That creature gains haste. Sacrifice the creature at the beginning of the next end step.")],
    delayed: &[delayed_trigger(cantrip_mechanics::delayed::sacrifice_subject, "Sacrifice the creature.")],
    ..oracle::SNEAK_ATTACK
};
