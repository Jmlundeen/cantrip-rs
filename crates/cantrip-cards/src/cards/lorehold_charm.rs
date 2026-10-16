use cantrip_api::*;
use cantrip_mechanics::{each_opponent_sacrifices, pump_creatures_you_control, targets, trample_until_end_of_turn};
use cantrip_oracle as oracle;

/// "A nontoken artifact" (CR 111.1): an artifact permanent that is a card, not a token.
fn nontoken_artifact(q: &dyn Query, id: ObjectId) -> bool {
    !q.obj(id).token && q.chars(id).is(CardType::Artifact)
}

/// "Each opponent sacrifices a nontoken artifact of their choice" (CR 701.21a, 101.4): in turn order, an opponent with one
/// sacrifices it without being asked, and one with none sacrifices nothing.
fn sacrifice_an_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    each_opponent_sacrifices(g, item.controller, item.object, 1, nontoken_artifact)
}

/// "Target artifact or creature card with mana value 2 or less from your graveyard" (CR 115.1, 202.3: X in a graveyard is
/// 0, the card's ruling).
fn artifact_or_creature_card(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard_with_mana_value_at_most(q, asker, t, &[CardType::Artifact, CardType::Creature], 2)
}

/// "Return target artifact or creature card with mana value 2 or less from your graveyard to the battlefield." With the
/// target gone or no longer such a card the spell does not resolve (CR 608.2b).
fn reanimate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    g.put_onto_battlefield(card, item.controller, false)?;
    Ok(())
}

/// "Creatures you control get +1/+1 and gain trample until end of turn": the creatures there as it resolves (CR 611.2c).
fn pump_and_trample(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for creature in pump_creatures_you_control(g, item.object, item.controller, 1, 1) {
        trample_until_end_of_turn(g, item.object, creature);
    }
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(sacrifice_an_artifact, "Each opponent sacrifices a nontoken artifact of their choice."),
    mode(reanimate, "Return target artifact or creature card with mana value 2 or less from your graveyard to the battlefield.")
        .targets(&[target(artifact_or_creature_card)]),
    mode(pump_and_trample, "Creatures you control get +1/+1 and gain trample until end of turn."),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::LOREHOLD_CHARM };
