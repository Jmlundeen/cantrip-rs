use cantrip_api::*;
use cantrip_mechanics::pump;
use cantrip_oracle as oracle;

/// The creatures attacking as the spell resolves get +2/+0 for the rest of the turn (CR 611.2c).
fn pump_attackers(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let attackers: Vec<ObjectId> = g.state().attackers.iter().map(|a| a.id)
        .filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    instruction(g, |g| {
        for id in attackers {
            pump(g, item.object, id, 2, 0);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], pump_attackers), ..oracle::ARMY_OF_ALLAH };
