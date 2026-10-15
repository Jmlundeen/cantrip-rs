use cantrip_api::*;
use cantrip_mechanics::library::ask_library_order;
use cantrip_mechanics::top_of_library;
use cantrip_oracle as oracle;

/// Look at the top five cards of your library. As many times as you choose, you may pay 1 life (so only while there is
/// life to pay, CR 119.4), put those cards on the bottom of your library in any order, then look at the top five cards.
/// Then shuffle and put the last cards you looked at on top in any order: the shuffle comes first, so they are held
/// as the cards they are and not read off the top again.
fn consult(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let mut looked = top_of_library(g, player, 5);
    if looked.is_empty() {
        return Ok(());
    }
    while g.state().player(player).life >= 1 {
        if !cantrip_mechanics::effects::ask_may(g, player, item.object, Purpose::Optional)? {
            break;
        }
        g.lose_life(player, 1);
        let bottom = ask_library_order(g, player, item.object, &looked, DestinationKind::LibraryBottom)?;
        g.arrange_library(player, &[], &bottom);
        looked = top_of_library(g, player, 5);
        if looked.is_empty() {
            return Ok(());
        }
    }
    g.shuffle(player);
    let top = ask_library_order(g, player, item.object, &looked, DestinationKind::LibraryTop)?;
    g.arrange_library(player, &top, &[]);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: Some(SpellDef { targets: &[], effect: consult }), ..oracle::LIM_D_LS_VAULT };
