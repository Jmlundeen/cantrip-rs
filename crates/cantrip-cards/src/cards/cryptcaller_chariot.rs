use cantrip_api::*;
use cantrip_mechanics::crew::crew;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::YOU_DISCARD;
use cantrip_oracle as oracle;

pub static EXTRA: &[&CardDef] = &[&tokens::ZOMBIE];

/// "Create that many tapped 2/2 black Zombie creature tokens": one for each card of the batch the trigger saw (CR 603.2c),
/// created together (CR 111.1), counted even for a card that has since left the graveyard.
fn that_many_zombies(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let many = g.event_objects(item).len() as u32;
    let zombie = g.card_id(&tokens::ZOMBIE);
    g.create_tokens(zombie, item.controller, many, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(YOU_DISCARD, that_many_zombies, "Whenever you discard one or more cards, create that many tapped 2/2 black Zombie creature tokens.")
            .once_per_batch(),
        crew(2, "Crew 2 (Tap any number of creatures you control with total power 2 or more: This Vehicle becomes an artifact creature until end of turn.)"),
    ],
    ..oracle::CRYPTCALLER_CHARIOT
};
