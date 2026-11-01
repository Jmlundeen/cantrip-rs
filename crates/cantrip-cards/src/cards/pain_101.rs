use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_DIES;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Until end of turn, target creature gains deathtouch and 'When this creature dies, return it to the battlefield tapped
/// under its owner's control'" (CR 611.2a, 613.1f): both abilities for the rest of the turn, the second the card's own
/// delayed ability. With the target gone nothing is gained (CR 608.2b).
fn gain_deathtouch_and_return(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.one_instruction(&mut |g| {
        effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Deathtouch);
        let this = g.card_id(&CARD);
        g.add_effect(item.object, creature, EffectKind::AddAbility(AbilityRef::delayed(this, 0)), Duration::EndOfTurn);
        Ok(())
    })
}

/// "Return it to the battlefield tapped under its owner's control" (CR 110.2a, 400.7): the card as it is in the graveyard
/// (the creature that died; each card of a merged permanent, together, CR 730.3c); a token is not a card and has ceased to exist,
/// so there is nothing to return (the rulings).
fn return_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::return_what_it_became(g, item.object, true, &[])?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], gain_deathtouch_and_return),
    delayed: &[triggered(THIS_DIES, return_it, "When this creature dies, return it to the battlefield tapped under its owner's control.")],
    ..oracle::PAIN_101
};
