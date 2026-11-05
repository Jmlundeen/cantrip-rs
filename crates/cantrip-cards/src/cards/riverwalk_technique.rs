use cantrip_api::*;
use cantrip_mechanics::{counter_target, library, targets};
use cantrip_oracle as oracle;

/// "Counter target noncreature spell." (CR 701.6)
fn counter_noncreature_spell(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target(g, item, 0);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(library::owner_puts_first_target_on_top_or_bottom, "The owner of target nonland permanent puts it on their choice of the top or bottom of their library.")
        .targets(&[targets::NONLAND_PERMANENT]),
    mode(counter_noncreature_spell, "Counter target noncreature spell.").targets(&[targets::NONCREATURE_SPELL]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::RIVERWALK_TECHNIQUE };
