use cantrip_api::*;
use cantrip_mechanics::{destroy_target, empower, targets};
use cantrip_oracle as oracle;

/// "You lose 2 life. Destroy target creature or planeswalker." The life is lost first (CR 608.2c); the mode's only target
/// being illegal as the spell resolves, none of it happens (CR 608.2b).
fn lose_life_and_destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.lose_life(item.controller, 2);
    destroy_target(g, item, 0);
    Ok(())
}

/// "You lose 2 life. Empower Jace 6." The Jace token is made if there is none, and six loyalty counters go on the one chosen
/// (the ruling: a token only, never a nontoken Jace).
fn lose_life_and_empower(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.lose_life(item.controller, 2);
    empower::jace(g, item.controller, item.object, 6)
}

static MODAL: ModalDef = modes(&[
    mode(lose_life_and_destroy, "You lose 2 life. Destroy target creature or planeswalker.").targets(&[targets::CREATURE_OR_PLANESWALKER]),
    mode(lose_life_and_empower, "You lose 2 life. Empower Jace 6."),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::VRASKAS_FINAL_MERCY };

pub static EXTRA: &[&CardDef] = &[&empower::JACE];
