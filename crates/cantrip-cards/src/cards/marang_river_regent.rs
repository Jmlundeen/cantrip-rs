use cantrip_api::*;
use cantrip_mechanics::{discard_chosen, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Draw three cards, then discard a card." (CR 121.1, 701.9): the card discarded is chosen once the three are drawn.
fn coil_and_catch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 3);
    discard_chosen(g, item.controller, item.object, 1)?;
    Ok(())
}

/// The Omen (CR 720): cast from the hand as an Omen instant for {3}{U}; as it resolves it is shuffled into its owner's
/// library (CR 720.3d, the core's).
pub static COIL_AND_CATCH: CardDef = CardDef {
    spell: spell(&[], coil_and_catch),
    ..oracle::COIL_AND_CATCH
};

/// "Return up to two other target nonland permanents to their owners' hands." Those still legal go together (CR 608.2b,
/// 400.7); none chosen, nothing happens.
fn return_them(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let returned = targets::legal_objects(g, item);
    g.return_to_hand(&returned);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        return_them,
        "When this creature enters, return up to two other target nonland permanents to their owners' hands.",
    )
    .targets(&[up_to(2, targets::other_nonland_permanent_filter)])],
    omen: Some(&COIL_AND_CATCH),
    ..oracle::MARANG_RIVER_REGENT
};

pub static EXTRA: &[&CardDef] = &[&COIL_AND_CATCH];
