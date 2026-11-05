use cantrip_api::*;
use cantrip_mechanics::{effects, mana, triggers::THIS_ENTERS_UNTAPPED};
use cantrip_oracle as oracle;

/// "This land deals 1 damage to each opponent." (CR 120.3, 102.2): one damage event for each opponent still in the game,
/// dealt together (CR 120.2).
fn damage_each_opponent(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::damage_each_opponent(g, item, 1);
    Ok(())
}

/// "When this land enters untapped, choose one —" (CR 603.6a, 700.2b).
static MODAL: ModalDef = modes(&[
    mode(damage_each_opponent, "This land deals 1 damage to each opponent."),
    mode(|g, item| effects::surveil(g, item, 1), "Surveil 1."),
]);

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped_unless_two_or_more_other_lands),
    abilities: &[
        modal_triggered(
            THIS_ENTERS_UNTAPPED,
            &MODAL,
            "When this land enters untapped, choose one —\n• This land deals 1 damage to each opponent.\n• Surveil 1.",
        ),
        mana_ability(TAP, add(Mana::R), "{T}: Add {R}."),
    ],
    ..oracle::QUARKS_BAR
};
