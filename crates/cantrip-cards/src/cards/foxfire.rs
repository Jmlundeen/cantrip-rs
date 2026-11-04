use cantrip_api::*;
use cantrip_mechanics::{effects, prevention, targets};
use cantrip_oracle as oracle;

/// Untap the target, prevent its combat damage, and schedule the next upkeep draw (CR 608.2c, 611.2a, 603.7b).
fn foxfire(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    prevention::untap_target_and_prevent_combat_damage_to_and_by_it(g, item)?;
    g.end_sentence();
    g.delayed_trigger(
        DelayedWhen::StepOfNextTurn { step: Step::Upkeep, of: None },
        item.object, item.controller, AbilityRef::delayed(item.ability.card, 0), Target::None,
    );
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ATTACKING_CREATURE], foxfire),
    delayed: &[delayed_trigger(|g, item| effects::draw(g, item, 1), "Draw a card at the beginning of the next turn's upkeep.")],
    granted_preventions: &[prevention::COMBAT_DAMAGE_DEALT_TO_AND_BY],
    ..oracle::FOXFIRE
};
