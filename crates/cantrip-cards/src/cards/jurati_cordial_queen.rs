use cantrip_api::*;
use cantrip_mechanics::copy::create_token_copy_of_target;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::creature_you_control_deals_combat_damage_to_player;
use cantrip_oracle as oracle;

/// "Whenever one or more artifact creatures you control deal combat damage to a player" (CR 510.2, 603.2c): an artifact
/// creature under Jurati's controller's control dealing combat damage to a player; one trigger for each player the batch
/// damaged, however many of the creatures dealt it.
fn artifact_creatures_you_control_deal_combat_damage_to_player(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { source, .. } if q.chars(*source).is(CardType::Artifact) => {
            creature_you_control_deals_combat_damage_to_player(q, me, event)
        }
        _ => None,
    }
}

/// "Target artifact token you control" (CR 115.1, 111.1): a token on the battlefield that the ability's controller controls
/// and that is an artifact as the layers make it now.
fn artifact_token_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && targets::token_you_control(q, asker, id) && q.chars(id).is(CardType::Artifact))
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::DAMAGE, artifact_creatures_you_control_deal_combat_damage_to_player),
        create_token_copy_of_target,
        "Whenever one or more artifact creatures you control deal combat damage to a player, create a token that's a copy of target artifact token you control.",
    )
    .sums_per_subject()
    .targets(&[target(artifact_token_you_control)])],
    ..oracle::JURATI_CORDIAL_QUEEN
};
