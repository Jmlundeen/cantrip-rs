use cantrip_api::*;
use cantrip_mechanics::escape;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "It deals 3 damage to any target and you gain 3 life."
fn bolt_and_gain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.target_legal(item, 0) {
        g.damage(item.object, item.targets[0], 3);
        g.gain_life(item.controller, 3);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        escape::sacrifice_unless_escaped("When Phlage enters, sacrifice it unless it escaped."),
        triggered(triggers::THIS_ENTERS_OR_ATTACKS, bolt_and_gain, "Whenever Phlage enters or attacks, it deals 3 damage to any target and you gain 3 life.")
            .targets(&[targets::ANY]),
    ],
    ways: &[escape::way(mana("{W}{W}{R}{R}"), 5, "Escape—{R}{R}{W}{W}, Exile five other cards from your graveyard")],
    ..oracle::PHLAGE_TITAN_OF_FIRES_FURY
};
