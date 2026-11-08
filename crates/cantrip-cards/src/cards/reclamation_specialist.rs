use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "{1}, Sacrifice an artifact creature: Draw a card." (CR 118.3, 602.2b): the {1} and the sacrifice are paid as the
/// ability is activated, so an unpayable cost draws nothing. The sacrificed permanent must be both an artifact and a
/// creature - a plain artifact or a plain creature cannot pay - and it may be this artifact creature itself, since the
/// card does not say "another". The engine's own "artifact creature" rule answers both tests (Argivian Blacksmith).
fn artifact_creature(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    (targets::ARTIFACT_CREATURE.filter)(q, asker, Target::Object(id))
}

/// "Draw a card" (CR 121.1): the ability's controller draws one card.
fn draw_for_the_cost(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{1}").and(sacrifice(1, artifact_creature)),
        draw_for_the_cost,
        "{1}, Sacrifice an artifact creature: Draw a card.",
    )],
    ..oracle::RECLAMATION_SPECIALIST
};
