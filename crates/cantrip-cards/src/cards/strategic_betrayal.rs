use cantrip_api::*;
use cantrip_mechanics::{choose, targets};
use cantrip_oracle as oracle;

/// "Target opponent exiles a creature they control and their graveyard." The opponent chooses the creature (asked even
/// when there is only one, none when there is none); the creature and every card in their graveyard are exiled together
/// (CR 701.13). The choice is made as the spell resolves (CR 608.2d), among the creatures they control then.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(victim) = item.target(0).player() else { return Ok(()) };
    let creatures: Vec<ObjectId> = g.permanents_of(victim).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    let mut exiled = g.state().zone(Zone::graveyard(victim)).to_vec();
    if let Some(creature) = choose::card(g, victim, item.object, Purpose::Exile, creatures)? {
        exiled.insert(0, creature);
    }
    g.exile_all(&exiled);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::OPPONENT], effect),
    ..oracle::STRATEGIC_BETRAYAL
};
