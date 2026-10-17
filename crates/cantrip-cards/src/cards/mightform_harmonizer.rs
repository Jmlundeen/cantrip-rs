use cantrip_api::*;
use cantrip_mechanics::{double, targets, triggers, warp};
use cantrip_oracle as oracle;

/// "Double the power of target creature you control until end of turn" (CR 701.10b-c): +X/+0 for the power it has as the
/// ability resolves. Nothing happens if the target is no longer legal (CR 608.2b).
fn double_power(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        double::power(g, item.object, creature);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when_asked(EventMask::ZONE_CHANGE, triggers::landfall), double_power, "Landfall \u{2014} Whenever a land you control enters, double the power of target creature you control until end of turn.")
        .targets(&[targets::CREATURE_YOU_CONTROL])],
    ways: &[warp::way(mana("{2}{G}"), "Warp {2}{G}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::MIGHTFORM_HARMONIZER
};
