use cantrip_api::*;
use cantrip_mechanics::crew::{becomes_a_creature, crew};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{craft, damage_target, targets};
use cantrip_oracle as oracle;

/// "When this artifact enters, it deals 5 damage to target tapped creature an opponent controls."
fn five_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 5);
    Ok(())
}

/// "Two other untapped artifacts you control": tapped to pay the cost (CR 118.3), so neither may be the Chariot itself.
fn other_untapped_artifact_you_control(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    id != asker.source && q.controller(id) == asker.controller && !q.obj(id).tapped && q.chars(id).is(CardType::Artifact)
}

pub static BACK: CardDef = CardDef {
    abilities: &[
        ability(Cost::tap_untapped(2, other_untapped_artifact_you_control), becomes_a_creature, "Tap two other untapped artifacts you control: This Vehicle becomes an artifact creature until end of turn."),
        crew(1, "Crew 1"),
    ],
    ..oracle::BLADEWHEEL_CHARIOT
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, five_damage, "When this artifact enters, it deals 5 damage to target tapped creature an opponent controls.")
            .targets(&[targets::TAPPED_CREATURE_AN_OPPONENT_CONTROLS]),
        craft::ability(mana("{3}{W}"), &craft::ARTIFACT, "Craft with artifact {3}{W}"),
    ],
    back: Some(&BACK),
    ..oracle::SPRING_LOADED_SAWBLADES
};
