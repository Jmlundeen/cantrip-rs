use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Counter target spell if it's blue.
fn counter_blue(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(spell) = targets::object(g, item, 0)
        && g.chars(spell).colors.has(Color::Blue)
    {
        g.counter(spell);
    }
    Ok(())
}

/// Destroy target permanent if it's blue.
fn destroy_blue(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(p) = targets::object(g, item, 0)
        && g.chars(p).colors.has(Color::Blue)
    {
        g.destroy(p);
    }
    Ok(())
}

static MODAL: ModalDef = ModalDef {
    modes: &[
        ModeDef { text: "Counter target spell if it's blue.", targets: &[targets::SPELL], effect: counter_blue, cost: ManaCost::ZERO },
        ModeDef {
            text: "Destroy target permanent if it's blue.",
            targets: &[targets::PERMANENT],
            effect: destroy_blue,
            cost: ManaCost::ZERO,
        },
    ],
    min: 1,
    max: 1,
    escalate: None,
    repeat: false, allowed: None,
};

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::PYROBLAST };
