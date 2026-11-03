use cantrip_api::*;
use cantrip_mechanics::{delayed, effects, prevention, targets};
use cantrip_oracle as oracle;

/// Prevent the next 1 damage to any target this turn, then set up the return (CR 615.7, 603.7).
fn prevent_next_1_damage_would(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    prevention::next_damage_to_target(g, item, 0, 0, 1);
    g.end_sentence();
    delayed::at_next_end_step(g, item, 0, Target::None);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{2}"),
        prevent_next_1_damage_would,
        "{2}: Prevent the next 1 damage that would be dealt to any target this turn. Return this artifact to its owner's hand at the beginning of the next end step.",
    ).targets(&[targets::ANY])],
    delayed: &[delayed_trigger(
        effects::return_this_to_hand,
        "Return this artifact to its owner's hand at the beginning of the next end step.",
    )],
    granted_preventions: &[prevention::ANY_DAMAGE.text("Prevent the next 1 damage that would be dealt to any target this turn.")],
    ..oracle::RAKALITE
};
