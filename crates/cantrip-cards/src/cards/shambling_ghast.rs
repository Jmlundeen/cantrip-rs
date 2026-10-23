use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_DIES;
use cantrip_mechanics::{effects, targets, tokens};
use cantrip_oracle as oracle;

/// Mode 2: "Create a Treasure token." (CR 111.10a.) The ability's controller, who controlled the Ghast as it died.
fn treasure(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_treasure(g, item.controller)
}

static MODAL: ModalDef = modes(&[
    mode(effects::target_creature_gets::<-1, -1>, "Target creature an opponent controls gets -1/-1 until end of turn.").targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS]),
    mode(treasure, "Create a Treasure token."),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(
        THIS_DIES,
        &MODAL,
        "When this creature dies, choose one \u{2014} Target creature an opponent controls gets -1/-1 until end of turn; or create a Treasure token.",
    )],
    ..oracle::SHAMBLING_GHAST
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
