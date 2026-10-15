use cantrip_api::*;
use cantrip_mechanics::library::{look_at_top, Rest};
use cantrip_oracle as oracle;

/// Look at the top three cards of your library. Put one into your hand and the rest on the bottom in any order; or,
/// if there is an instant card and a sorcery card in your graveyard as this resolves, two of them instead. The
/// graveyard is read while Flow State is still on the stack. A library shorter than the hand count puts them all there.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let graveyard = g.state().zone(Zone::graveyard(player)).to_vec();
    let both = graveyard.iter().any(|&c| g.chars(c).is(CardType::Instant))
        && graveyard.iter().any(|&c| g.chars(c).is(CardType::Sorcery));
    let taken = if both { 2 } else { 1 };
    look_at_top(g, player, item.object, 3, (taken, taken), false, |_| true, Rest::BottomChosen)?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: Some(SpellDef { targets: &[], effect }), ..oracle::FLOW_STATE };
