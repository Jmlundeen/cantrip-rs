use cantrip_api::*;
use cantrip_oracle as oracle;

/// "an artifact or land" as an additional sacrifice cost.
fn artifact_or_land(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
    let c = q.chars(id);
    c.is(CardType::Artifact) || c.is(CardType::Land)
}

fn draw_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{2}").and(TAP).and(Cost::sacrifice(1, artifact_or_land)),
        draw_a_card,
        "{2}, {T}, Sacrifice an artifact or land: Draw a card.",
    )],
    ..oracle::MARWYN_THE_CLEARCUTTER
};
