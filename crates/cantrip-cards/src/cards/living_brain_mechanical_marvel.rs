use cantrip_api::*;
use cantrip_mechanics::triggers::BEGINNING_OF_YOUR_COMBAT;
use cantrip_mechanics::{Animation, animate, targets};
use cantrip_oracle as oracle;

/// "Target non-Equipment artifact you control" (CR 115.1, 301.5): an artifact on the battlefield under the ability's controller
/// that is not an Equipment: an Equipment that is also a creature is still an Equipment (CR 301.5).
fn non_equipment_artifact_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(
        t,
        Target::Object(id) if targets::permanent(q, id)
            && q.chars(id).is(CardType::Artifact)
            && !q.chars(id).has_subtype(Subtype::Equipment)
            && q.controller(id) == asker.controller
    )
}

/// "Target non-Equipment artifact you control becomes an artifact creature with base power and toughness 3/3 until end of turn.
/// Untap it." (CR 205.1b, 613.4b, 701.26): it keeps its card types and subtypes and gains Creature (layer 4), its base power and
/// toughness are set to 3/3 (7b, so counters and pumps count on top), and it is untapped afterwards. A target no longer legal
/// as the ability resolves does nothing (CR 608.2b).
fn animate_and_untap(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(artifact) = targets::object(g, item, 0) else { return Ok(()) };
    let animation = Animation {
        power: 3,
        toughness: 3,
        colors: None,
        types: TypeSet::of(&[CardType::Artifact]),
        subtypes: &[],
        all_creature_types: false,
        replaces_creature_types: false,
        keywords: Keywords::NONE,
        ability: None,
    };
    animate(g, item.object, artifact, &animation, Duration::EndOfTurn);
    g.end_sentence();
    g.untap(artifact);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        BEGINNING_OF_YOUR_COMBAT,
        animate_and_untap,
        "At the beginning of combat on your turn, target non-Equipment artifact you control becomes an artifact creature with base power and toughness 3/3 until end of turn. Untap it.",
    )
    .targets(&[target(non_equipment_artifact_you_control)])],
    ..oracle::LIVING_BRAIN_MECHANICAL_MARVEL
};
