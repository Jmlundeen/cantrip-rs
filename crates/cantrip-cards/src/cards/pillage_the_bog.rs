use cantrip_api::*;
use cantrip_mechanics::library::{Rest, look_at_top};
use cantrip_mechanics::plot::plot;
use cantrip_oracle as oracle;

/// Look at the top X cards of your library, where X is twice the number of lands you control as this resolves. Put one of
/// them into your hand and the rest on the bottom of your library in a random order. No lands is looking at nothing, which
/// does nothing (CR 609.3).
fn look(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let lands = g.permanents_of(player).into_iter().filter(|&id| g.chars(id).is(CardType::Land)).count();
    look_at_top(g, player, item.object, lands * 2, (1, 1), false, |_| true, Rest::BottomRandom)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect: look }),
    abilities: &[plot(
        mana(1, 0, 0, 1, 0, 1),
        "Plot {1}{B}{G} (You may pay {1}{B}{G} and exile this card from your hand. Cast it as a sorcery on a later turn without paying its mana cost. Plot only as a sorcery.)",
    )],
    ..oracle::PILLAGE_THE_BOG
};
