use cantrip_api::*;
use cantrip_mechanics::flashback;
use cantrip_mechanics::shuffle_hands_and_graveyards_in_and_draw_seven;
use cantrip_oracle as oracle;

fn effect(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    shuffle_hands_and_graveyards_in_and_draw_seven(g);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    ways: &[flashback::way(mana(2, 0, 1, 0, 0, 0), "Flashback {2}{U}")],
    ..oracle::ECHO_OF_EONS
};
