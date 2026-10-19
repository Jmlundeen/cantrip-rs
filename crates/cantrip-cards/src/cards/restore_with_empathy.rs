use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Return target permanent card from your graveyard to your hand. You gain 4 life." (CR 400.3, 119.3): the card returns
/// if its target is still legal (CR 608.2b), and the life is gained only when the spell resolves; an illegal sole target stops the entire spell (CR 608.2b).
fn restore(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::return_first_target_to_hand(g, item)?;
    g.gain_life(item.controller, 4);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PERMANENT_CARD_IN_YOUR_GRAVEYARD], restore),
    ..oracle::RESTORE_WITH_EMPATHY
};
