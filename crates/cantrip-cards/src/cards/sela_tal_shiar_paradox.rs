use cantrip_api::*;
use cantrip_mechanics::{effects, targets, tokens, triggers};
use cantrip_oracle as oracle;

/// "Create a tapped 2/2 black Romulan creature token." (CR 701.7a, 111.2, 612.2a)
fn create_romulan(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let romulan = g.card_id(&tokens::ROMULAN);
    g.create_tokens(romulan, item.controller, 1, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ATTACKS, effects::exile_first_target, "Whenever Sela attacks, exile up to one target card from a graveyard.")
            .targets(&[targets::CARD_IN_A_GRAVEYARD.optional()]),
        triggered(
            triggers::CARDS_LEAVE_YOUR_GRAVEYARD,
            create_romulan,
            "Whenever one or more cards leave your graveyard, create a tapped 2/2 black Romulan creature token.",
        )
        .once_per_batch(),
    ],
    ..oracle::SELA_TAL_SHIAR_PARADOX
};

pub static EXTRA: &[&CardDef] = &[&tokens::ROMULAN];
