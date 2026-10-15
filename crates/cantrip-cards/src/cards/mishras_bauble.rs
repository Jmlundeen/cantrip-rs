use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Look at the top card of target player's library; draw a card at the beginning of the next turn's upkeep. Nothing
/// happens if the player is no longer a legal target (CR 608.2b). The look changes no game state, so it is not
/// modelled; the delayed draw is set up whether or not that library has a card (CR 609.3).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let card = g.obj(item.object).card;
    g.delayed_trigger(
        DelayedWhen::Step { step: Step::Upkeep, on_turn_of: None },
        item.object,
        item.controller,
        AbilityRef::delayed(card, 0),
        Target::None,
    );
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{T}, Sacrifice Mishra's Bauble: Look at the top card of target player's library. Draw a card at the beginning of the next turn's upkeep.",
        cost: Cost { tap: true, sacrifice_self: true, ..Cost::NONE },
        targets: &[targets::PLAYER],
        effect,
        ..ActivatedDef::BLANK
    })],
    delayed: &[AbilityDef::Triggered(TriggeredDef { text: "Draw a card.", effect: effects::draw::<1>, ..TriggeredDef::BLANK })],
    ..oracle::MISHRAS_BAUBLE
};
