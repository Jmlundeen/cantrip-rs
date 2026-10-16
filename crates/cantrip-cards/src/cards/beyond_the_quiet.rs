use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Exile all creatures and Spacecraft." Every creature and every Spacecraft on the battlefield, whoever controls it,
/// is chosen as the spell resolves and exiled at once (CR 406.2, 608.2c, 101.4): a Vehicle is a creature only while it is
/// crewed, and a Spacecraft is exiled whether or not it is a creature. Nothing is destroyed, so indestructible and
/// regeneration do not help, and nothing dies.
fn effect(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let all: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| {
            let chars = g.chars(id);
            chars.is(CardType::Creature) || chars.has_subtype(Subtype::Spacecraft)
        })
        .collect();
    g.exile_all(&all);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ..oracle::BEYOND_THE_QUIET
};
