use cantrip_api::*;
use cantrip_mechanics::fight::bite;
use cantrip_mechanics::library::{Find, search_land_tapped_else_hand};
use cantrip_mechanics::{exile_target, targets};
use cantrip_oracle as oracle;

/// "Search your library for a creature or land card and reveal it. Put it onto the battlefield tapped if it's a land card.
/// Otherwise, put it into your hand. Then shuffle." (CR 701.23): the card may be left unfound (CR 701.23b); a card that is
/// both a land and a creature is a land card, so it is put onto the battlefield tapped. The library is shuffled either way.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_land_tapped_else_hand(g, item.controller, item.object, Find::ONE.revealed(), |q, c| {
        let chars = q.chars(c);
        chars.is(CardType::Creature) || chars.is(CardType::Land)
    })?;
    Ok(())
}

/// "Put a +1/+1 counter on target creature you control. It deals damage equal to its power to target creature you don't
/// control." (CR 608.2b): each target is checked as it resolves. If the creature you control is no longer a legal target
/// the counter is not put and no damage is dealt; if only the other creature is illegal, the counter is still put. The
/// damage is read after the counter, by the creature as the source (`fight::bite`).
fn counter_and_bite(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(creature, CounterKind::PlusOne, 1);
    if let Some(victim) = targets::object(g, item, 1) {
        bite(g, creature, victim);
    }
    Ok(())
}

/// "Exile target artifact or enchantment."
fn exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(search, "Search your library for a creature or land card and reveal it. Put it onto the battlefield tapped if it's a land card. Otherwise, put it into your hand. Then shuffle."),
    mode(counter_and_bite, "Put a +1/+1 counter on target creature you control. It deals damage equal to its power to target creature you don't control.")
        .targets(&[targets::CREATURE_YOU_CONTROL, targets::CREATURE_YOU_DONT_CONTROL]),
    mode(exile, "Exile target artifact or enchantment.").targets(&[targets::ARTIFACT_OR_ENCHANTMENT]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::ARCHDRUIDS_CHARM };
