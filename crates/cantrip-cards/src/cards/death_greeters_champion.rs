use cantrip_api::*;
use cantrip_mechanics::backup;
use cantrip_mechanics::dash;
use cantrip_oracle as oracle;

/// Backup 1 (CR 702.157a): put a +1/+1 counter on target creature; if that's another creature, it gains double strike.
fn backup(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    backup::resolve(g, item, 1, &[Keyword::DoubleStrike], None)
}

pub static CARD: CardDef = CardDef {
    abilities: &[backup::trigger("When this creature enters, put a +1/+1 counter on target creature. If that's another creature, it gains double strike until end of turn.", backup)],
    ways: &[dash::way(mana("{3}{R}"), "Dash {3}{R}")],
    end_step_riders: dash::RIDERS,
    delayed: &[dash::RETURN],
    statics: &[dash::HASTE],
    ..oracle::DEATH_GREETERS_CHAMPION
};
