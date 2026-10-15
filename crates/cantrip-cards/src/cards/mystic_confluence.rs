use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::{counter_unless_pays, targets};
use cantrip_oracle as oracle;

/// Counter target spell unless its controller pays {3} (CR 118.12).
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_unless_pays(g, item, 0, mana(3, 0, 0, 0, 0, 0))?;
    Ok(())
}

/// Return target creature to its owner's hand.
fn bounce(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.return_to_hand(&[creature]);
    }
    Ok(())
}

/// Choose three; you may choose the same mode more than once (CR 700.2d).
static MODAL: ModalDef = ModalDef {
    modes: &[
        ModeDef {
            text: "Counter target spell unless its controller pays {3}.",
            targets: &[targets::SPELL],
            effect: counter,
            cost: ManaCost::ZERO,
        },
        ModeDef {
            text: "Return target creature to its owner's hand.",
            targets: &[targets::CREATURE],
            effect: bounce,
            cost: ManaCost::ZERO,
        },
        ModeDef { text: "Draw a card.", targets: &[], effect: effects::draw::<1>, cost: ManaCost::ZERO },
    ],
    min: 3,
    max: 3,
    escalate: None,
    repeat: true, allowed: None,
};

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::MYSTIC_CONFLUENCE };
