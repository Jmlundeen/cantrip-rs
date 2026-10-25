use cantrip_api::*;
use cantrip_mechanics::{targets, tokens};
use cantrip_oracle as oracle;

/// "Destroy target artifact. If you controlled that artifact, create three 1/1 red Phyrexian Goblin creature tokens."
/// Whether you controlled it is read as the artifact was before it is destroyed (CR 608.2h); the tokens are made whether or
/// not the destruction happened (an indestructible artifact you control still counts).
fn demolish(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(artifact) = targets::object(g, item, 0) else { return Ok(()) };
    let yours = g.controller(artifact) == item.controller;
    g.destroy(artifact);
    g.end_sentence();
    if yours {
        tokens::create(g, item.controller, &tokens::PHYREXIAN_GOBLIN, 3)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ARTIFACT], demolish),
    ..oracle::GLEEFUL_DEMOLITION
};

pub static EXTRA: &[&CardDef] = &[&tokens::PHYREXIAN_GOBLIN];
