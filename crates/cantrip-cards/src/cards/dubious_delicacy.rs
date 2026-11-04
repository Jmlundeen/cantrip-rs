use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target opponent loses 3 life." (CR 119.3.) A target no longer legal loses nothing (CR 608.2b).
fn opponent_loses_3(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(player) = targets::player(g, item, 0) {
        g.lose_life(player, 3);
    }
    Ok(())
}

/// Flash is a core keyword. "When this artifact enters, up to one target creature gets -3/-3 until end of turn" (CR 611.2a): an
/// up-to-one target, so the trigger is put on the stack with none when there is no creature, or none is chosen. Its two
/// activated abilities share the cost "{2}, {T}, Sacrifice this artifact" and each is its own ability (CR 111.10b for the Food one);
/// the second has a target opponent.
pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, effects::target_creature_gets::<-3, -3>, "When this artifact enters, up to one target creature gets -3/-3 until end of turn.")
            .targets(&[targets::CREATURE.optional()]),
        ability(cost("{2}, {T}").and(SACRIFICE_THIS), effects::gain_three_life, "{2}, {T}, Sacrifice this artifact: You gain 3 life."),
        ability(cost("{2}, {T}").and(SACRIFICE_THIS), opponent_loses_3, "{2}, {T}, Sacrifice this artifact: Target opponent loses 3 life.")
            .targets(&[targets::OPPONENT]),
    ],
    ..oracle::DUBIOUS_DELICACY
};
