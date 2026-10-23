use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::OPPONENT_ACTIVATES_ABILITY_OF_ARTIFACT_CREATURE_OR_LAND;
use cantrip_mechanics::pump;
use cantrip_oracle as oracle;

/// "{3}{R}{R}: This creature gets +3/+3 and gains menace until end of turn" (CR 611.2, 702.111).
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    instruction(g, |g| {
        pump(g, item.object, item.object, 3, 3);
        effects::gain_until_end_of_turn(g, item.object, item.object, Keyword::Menace);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            OPPONENT_ACTIVATES_ABILITY_OF_ARTIFACT_CREATURE_OR_LAND,
            |g, item| effects::damage_that_player(g, item, 1),
            "Whenever an opponent activates an ability of an artifact, creature, or land that isn't a mana ability, this creature deals 1 damage to that player.",
        ),
        ability(cost("{3}{R}{R}"), grow, "{3}{R}{R}: This creature gets +3/+3 and gains menace until end of turn."),
    ],
    ..oracle::IMMOLATION_SHAMAN
};
