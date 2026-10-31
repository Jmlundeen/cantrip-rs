use cantrip_api::*;
use cantrip_mechanics::{effects, lands, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When this land enters, up to one target creature phases out." (CR 702.26b.) It phases in during its controller's next
/// untap step (CR 502.1). With no target chosen, nothing happens.
fn phase_out_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.phase_out(&[creature]);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, phase_out_target, "When this land enters, up to one target creature phases out.")
            .targets(&[targets::CREATURE.optional()]),
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        lands::ANY_COLOR_FOR_ONE,
        ability(cost("{4}"), effects::return_this_card_to_battlefield, "{4}: Put this card from your hand onto the battlefield.").from(ZoneKind::Hand),
    ],
    ..oracle::TALON_GATES_OF_MADARA
};
