use cantrip_api::*;
use cantrip_mechanics::{flurry, tokens};
use cantrip_oracle as oracle;

/// "Create a 1/1 white Bird creature token with flying" (CR 111.1, 111.4).
fn create_bird(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::BIRD, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[flurry::trigger("Flurry \u{2014} Whenever you cast your second spell each turn, create a 1/1 white Bird creature token with flying.", create_bird)],
    ..oracle::WINGBLADE_DISCIPLE
};

pub static EXTRA: &[&CardDef] = &[&tokens::BIRD];
