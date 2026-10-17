use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, pump, targets};
use cantrip_oracle as oracle;

/// "Target creature an opponent controls gets -2/-2 until end of turn." Nothing if the creature is no longer a legal
/// target as the trigger resolves (CR 608.2b).
fn shrink_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        pump(g, item.object, creature, -2, -2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, shrink_target, "When this artifact enters, target creature an opponent controls gets -2/-2 until end of turn.")
            .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS]),
        ability(
            cost("{2}{B}, {T}").and(SACRIFICE_THIS),
            |g, item| effects::surveil(g, item, 2),
            "{2}{B}, {T}, Sacrifice this artifact: Surveil 2.",
        ),
    ],
    ..oracle::GRIM_BAUBLE
};
