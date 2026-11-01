use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Tap up to one target creature and put a stun counter on it. Draw a card." (CR 701.26a, 122.1d, 121.1)
fn observe(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::tap_and_stun::<1>(g, item)?;
    g.draw(item.controller, 1);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(effects::counter_first_target_unless_pays::<3>, "Interfere — Counter target spell unless its controller pays {3}.")
        .targets(&[targets::SPELL]),
    mode(observe, "Observe — Tap up to one target creature and put a stun counter on it. Draw a card.")
        .targets(&[targets::CREATURE.optional()]),
]);

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&MODAL),
    ..oracle::CONSIDER_THE_PRIME_DIRECTIVE
};
