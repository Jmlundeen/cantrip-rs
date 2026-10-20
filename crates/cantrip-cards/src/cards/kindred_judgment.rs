use cantrip_api::*;
use cantrip_mechanics::chosen_type;
use cantrip_oracle as oracle;

/// "Choose a creature type. Destroy all creatures that aren't of the chosen type." The type is chosen as the spell
/// resolves (CR 608.2d); the creatures are judged after (CR 608.2c) and destroyed together (CR 701.8a).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let chosen = chosen_type::creature_type_now(g, you, item.object)?;
    let doomed: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| {
            let c = g.chars(id);
            c.is(CardType::Creature) && !c.has_subtype(chosen)
        })
        .collect();
    g.destroy_all(&doomed);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ..oracle::KINDRED_JUDGMENT
};
