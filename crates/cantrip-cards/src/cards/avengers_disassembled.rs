use cantrip_api::*;
use cantrip_mechanics::lands::destroy_target_its_controller_may_fetch_basic;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Mode 0: "Avengers Disassembled deals 3 damage to each creature." One event for every creature (CR 120.2, 603.2c),
/// the creatures being those on the battlefield as the mode is carried out (CR 608.2c).
fn damage_creatures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::damage_each_creature(g, item, 3);
    Ok(())
}

/// Mode 1: "Destroy target land. Its controller may search their library for a basic land card, put it onto the
/// battlefield tapped, then shuffle."
fn destroy_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target_its_controller_may_fetch_basic(g, item, 0, true)
}

/// "Choose one or both" (CR 700.2): the modes are carried out in the order printed (CR 700.2d).
static MODAL: ModalDef = modes(&[
    mode(damage_creatures, "Avengers Disassembled deals 3 damage to each creature."),
    mode(destroy_land, "Destroy target land. Its controller may search their library for a basic land card, put it onto the battlefield tapped, then shuffle.")
        .targets(&[targets::LAND]),
])
    .choose(1, 2);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::AVENGERS_DISASSEMBLED };
