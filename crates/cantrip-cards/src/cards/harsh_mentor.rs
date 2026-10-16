use cantrip_api::*;
use cantrip_mechanics::triggers::OPPONENT_ACTIVATES_ABILITY_OF_ARTIFACT_CREATURE_OR_LAND;
use cantrip_oracle as oracle;

/// "This creature deals 2 damage to that player": the player who activated the ability (the trigger's subject), dealt by
/// this creature, as it last was if it has left (CR 113.7a). The ability resolves before the activated one (the ruling).
fn two_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Player(p) = item.subject {
        g.damage(item.object, Target::Player(p), 2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        OPPONENT_ACTIVATES_ABILITY_OF_ARTIFACT_CREATURE_OR_LAND,
        two_damage,
        "Whenever an opponent activates an ability of an artifact, creature, or land on the battlefield, if it isn't a mana ability, this creature deals 2 damage to that player.",
    )],
    ..oracle::HARSH_MENTOR
};
