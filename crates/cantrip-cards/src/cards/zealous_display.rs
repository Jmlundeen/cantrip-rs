use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// Creatures you control get +2/+0 until end of turn (the creatures fixed as it resolves, CR 611.2c); if it's not your turn,
/// untap those creatures.
fn pump_and_untap_off_turn(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures = effects::pump_creatures_you_control(g, item, 2, 0);
    g.end_sentence();
    if g.state().turn.active != item.controller {
        for &creature in &creatures {
            g.untap(creature);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], pump_and_untap_off_turn),
    ..oracle::ZEALOUS_DISPLAY
};
