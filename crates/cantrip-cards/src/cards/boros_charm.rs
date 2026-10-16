use cantrip_api::*;
use cantrip_mechanics::effects::gain_indestructible_until_end_of_turn;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "Boros Charm deals 4 damage to target player or planeswalker": nothing if the target is no longer legal (CR 608.2b).
fn four_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 4);
    Ok(())
}

/// "Permanents you control gain indestructible until end of turn": the permanents there are as it resolves, and only those
/// (CR 611.2c).
fn indestructible(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for permanent in g.permanents_of(item.controller) {
        gain_indestructible_until_end_of_turn(g, item.object, permanent);
    }
    Ok(())
}

/// "Target creature gains double strike until end of turn": nothing if the target is no longer legal (CR 608.2b).
fn double_strike(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::DoubleStrike])), Duration::EndOfTurn);
    }
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(four_damage, "Boros Charm deals 4 damage to target player or planeswalker.").targets(&[targets::PLAYER_OR_PLANESWALKER]),
    mode(indestructible, "Permanents you control gain indestructible until end of turn."),
    mode(double_strike, "Target creature gains double strike until end of turn.").targets(&[targets::CREATURE]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::BOROS_CHARM };
