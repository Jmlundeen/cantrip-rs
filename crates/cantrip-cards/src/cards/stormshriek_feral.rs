use cantrip_api::*;
use cantrip_mechanics::{discard_chosen, effects};
use cantrip_oracle as oracle;

/// "Discard a card. If you do, draw two cards" (CR 701.9, 608.2c): the controller chooses a card of their hand and discards it, an
/// empty hand being asked nothing; the draws follow a card actually discarded.
fn discard_to_draw_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !discard_chosen(g, item.controller, item.object, 1)?.is_empty() {
        g.draw(item.controller, 2);
    }
    Ok(())
}

/// The Omen (CR 720): cast from the hand as an Omen sorcery for {1}{R}; as it resolves it is shuffled into its owner's library
/// (CR 720.3d, the core's).
pub static FLUSH_OUT: CardDef = CardDef { spell: spell(&[], discard_to_draw_two), ..oracle::FLUSH_OUT };

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{1}{R}"), effects::this_gets::<1, 0>, "{1}{R}: This creature gets +1/+0 until end of turn.")],
    omen: Some(&FLUSH_OUT),
    ..oracle::STORMSHRIEK_FERAL
};

pub static EXTRA: &[&CardDef] = &[&FLUSH_OUT];
