use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// Destroy target creature.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    // As an additional cost to cast this spell, pay {4} or sacrifice an artifact or creature (CR 601.2b).
    additional_choice: &[
        ExtraCost { text: "Pay {4}", cost: Cost { mana: mana("{4}"), ..Cost::NONE } },
        ExtraCost { text: "Sacrifice an artifact or creature", cost: Cost::sacrifice(1, targets::filters::artifact_or_creature) },
    ],
    ..oracle::DEADLY_PRECISION
};
