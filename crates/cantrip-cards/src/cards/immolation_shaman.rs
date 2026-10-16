use cantrip_api::*;
use cantrip_mechanics::triggers::OPPONENT_ACTIVATES_ABILITY_OF_ARTIFACT_CREATURE_OR_LAND;
use cantrip_mechanics::pump;
use cantrip_oracle as oracle;

/// "This creature deals 1 damage to that player": the player who activated the ability (the trigger's subject), dealt by
/// this creature, as it last was if it has left (CR 113.7a). The ability resolves before the activated one (the ruling).
fn one_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Player(p) = item.subject {
        g.damage(item.object, Target::Player(p), 1);
    }
    Ok(())
}

/// "{3}{R}{R}: This creature gets +3/+3 and gains menace until end of turn" (CR 611.2, 702.111).
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    pump(g, item.object, item.object, 3, 3);
    g.add_effect(item.object, item.object, EffectKind::AddKeywords(Keywords::of(&[Keyword::Menace])), Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            OPPONENT_ACTIVATES_ABILITY_OF_ARTIFACT_CREATURE_OR_LAND,
            one_damage,
            "Whenever an opponent activates an ability of an artifact, creature, or land that isn't a mana ability, this creature deals 1 damage to that player.",
        ),
        ability(cost("{3}{R}{R}"), grow, "{3}{R}{R}: This creature gets +3/+3 and gains menace until end of turn."),
    ],
    ..oracle::IMMOLATION_SHAMAN
};
