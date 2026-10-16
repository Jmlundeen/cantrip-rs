use cantrip_api::*;
use cantrip_mechanics::{effects::damage_each_creature, tiered};
use cantrip_oracle as oracle;

/// Fire: "Fire Magic deals 1 damage to each creature." Every creature on the battlefield as it resolves, at once.
fn fire(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_each_creature(g, item, 1);
    Ok(())
}

/// Fira: "Fire Magic deals 2 damage to each creature."
fn fira(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_each_creature(g, item, 2);
    Ok(())
}

/// Firaga: "Fire Magic deals 3 damage to each creature."
fn firaga(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_each_creature(g, item, 3);
    Ok(())
}

const MODES: &[ModeDef] = &[
    mode(fire, "Fire — {0} — Fire Magic deals 1 damage to each creature.").cost(mana("{0}")),
    mode(fira, "Fira — {2} — Fire Magic deals 2 damage to each creature.").cost(mana("{2}")),
    mode(firaga, "Firaga — {5} — Fire Magic deals 3 damage to each creature.").cost(mana("{5}")),
];

static MODAL: ModalDef = tiered::modal(MODES);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::FIRE_MAGIC };
