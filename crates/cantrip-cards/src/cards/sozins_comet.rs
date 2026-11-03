use cantrip_api::*;
use cantrip_mechanics::{firebending, foretell};
use cantrip_oracle as oracle;

/// "Each creature you control gains firebending 5 until end of turn" (CR 611.2c, 613.1f, 702.189a): the creatures its controller
/// controls as it resolves each get the card's delayed ability 0, a firebending trigger of their own, until the cleanup step. A
/// creature that arrives later never gains it.
fn grant(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let this = g.card_id(&CARD);
    let creatures: Vec<ObjectId> = g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    instruction(g, |g| {
        for creature in creatures {
            g.add_effect(item.object, creature, EffectKind::AddAbility(AbilityRef::delayed(this, 0)), Duration::EndOfTurn);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], grant),
    foretell: Some(mana("{2}{R}")),
    abilities: &[foretell::ABILITY],
    // The granted ability (the card's delayed ability 0).
    delayed: &[firebending::trigger::<5>("Firebending 5 (Whenever this creature attacks, add {R}{R}{R}{R}{R}. This mana lasts until end of combat.)")],
    ..oracle::SOZINS_COMET
};
