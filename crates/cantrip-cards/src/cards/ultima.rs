use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Destroy all artifacts and creatures. End the turn." Every artifact and creature goes at once (CR 701.8a, 101.4), then
/// the turn ends (CR 724.1): the abilities their leaving triggered cease to exist with everything else waiting (724.1a),
/// and Ultima is exiled from the stack with every other spell and ability on it (724.1b).
fn effect(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let doomed: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| {
            let c = g.chars(id);
            c.is(CardType::Artifact) || c.is(CardType::Creature)
        })
        .collect();
    g.destroy_all(&doomed);
    g.end_the_turn()
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::ULTIMA };
