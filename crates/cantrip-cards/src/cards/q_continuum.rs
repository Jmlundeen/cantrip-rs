use cantrip_api::*;
use cantrip_mechanics::{effects, mana, targets, triggers::THIS_ENTERS_UNTAPPED};
use cantrip_oracle as oracle;

/// "Target creature gets +1/+1 and gains vigilance until end of turn." (CR 611.2a, 702.20): one instruction, the creature
/// being the mode's target; nothing if it is no longer a legal target as the ability resolves (CR 608.2b).
fn pump_and_vigilance(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    instruction(g, |g| {
        cantrip_mechanics::pump(g, item.object, creature, 1, 1);
        effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Vigilance);
    });
    Ok(())
}

/// "When this land enters untapped, choose one —" (CR 603.6a, 700.2b).
static MODAL: ModalDef = modes(&[
    mode(pump_and_vigilance, "Target creature gets +1/+1 and gains vigilance until end of turn.").targets(&[targets::CREATURE]),
    mode(|g, item| effects::surveil(g, item, 1), "Surveil 1."),
]);

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped_unless_two_or_more_other_lands),
    abilities: &[
        modal_triggered(
            THIS_ENTERS_UNTAPPED,
            &MODAL,
            "When this land enters untapped, choose one —\n• Target creature gets +1/+1 and gains vigilance until end of turn.\n• Surveil 1.",
        ),
        mana_ability(TAP, add(Mana::G), "{T}: Add {G}."),
    ],
    ..oracle::Q_CONTINUUM
};
