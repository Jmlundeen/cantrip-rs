use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "All nonartifact creatures get -X/-X until end of turn, where X is the number of artifact creatures you control"
/// (CR 608.2h, 611.2c): X is counted once, as the spell resolves, among the artifact creatures its controller controls, and
/// stays that amount from then on (CR 611.2a). Every creature that is not an artifact, whoever controls it, is pumped, one
/// instruction for all of them; an artifact creature is left alone even when it is also a creature.
fn resolve(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let x = g
        .permanents_of(you)
        .into_iter()
        .filter(|&id| {
            let chars = g.chars(id);
            chars.is(CardType::Artifact) && chars.is(CardType::Creature)
        })
        .count() as i32;
    // The pump visits the creatures on the battlefield, so the filter only has to leave out the artifact ones.
    effects::pump_creatures_where(g, item.object, |q, id| !q.chars(id).is(CardType::Artifact), -x, -x);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], resolve),
    ..oracle::BATTLE_OF_WOLF_359
};
