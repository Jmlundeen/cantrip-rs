use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

fn deal_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 2);
    Ok(())
}

fn another_artifact(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    id != asker.source && q.chars(id).is(CardType::Artifact) && q.controller(id) == asker.controller
}

fn golem(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::GOLEM, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, deal_two, "When this artifact enters, it deals 2 damage to any target.")
            .targets(&[targets::ANY]),
        ability(cost("{2}, {T}").and(sacrifice(1, another_artifact)), golem, "{2}, {T}, Sacrifice another artifact: Create a 3/3 colorless Golem artifact creature token."),
    ],
    ..oracle::LEGION_EXTRUDER
};

pub static EXTRA: &[&CardDef] = &[&tokens::GOLEM];
