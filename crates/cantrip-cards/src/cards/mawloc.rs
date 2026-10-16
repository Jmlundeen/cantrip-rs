use cantrip_api::*;
use cantrip_mechanics::ravenous;
use cantrip_mechanics::replacements::{EXILE_INSTEAD_OF_DYING, exile_if_it_would_die};
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// It fights up to one target creature an opponent controls. If that creature would die this turn, exile it instead. The
/// replacement is the ability's own effect on that creature, made whether or not the fight happens; with Mawloc no longer
/// a creature on the battlefield, neither fights (CR 701.12b).
fn terror_from_the_deep(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if item.target_count == 0 || !g.target_legal(item, 0) {
        return Ok(());
    }
    let Target::Object(prey) = item.targets[0] else { return Ok(()) };
    exile_if_it_would_die(g, item.object, 0, prey);
    g.fight(item.object, prey);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(ravenous::enters),
    abilities: &[
        ravenous::draw_trigger("When Mawloc enters, if X is 5 or more, draw a card."),
        triggered(THIS_ENTERS, terror_from_the_deep, "Terror from the Deep \u{2014} When Mawloc enters, it fights up to one target creature an opponent controls. If that creature would die this turn, exile it instead.")
            .targets(&[TargetSpec { min: 0, max: 1, filter: targets::CREATURE_AN_OPPONENT_CONTROLS.filter }]),
    ],
    granted_replacements: &[EXILE_INSTEAD_OF_DYING],
    ..oracle::MAWLOC
};
