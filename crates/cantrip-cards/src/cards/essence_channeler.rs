use cantrip_api::*;
use cantrip_mechanics::effects::plus_one_counter_on_this;
use cantrip_mechanics::statics::itself;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::{THIS_DIES, YOU_GAIN_LIFE};
use cantrip_oracle as oracle;

/// "As long as you've lost life this turn": whether this creature's controller has lost any life this turn (CR 119.3), however
/// much they have gained besides (the ruling). Asked of the event history each time the layers apply (CR 611.3a), so the
/// abilities come the moment the life is lost and go as the next turn begins.
fn you_lost_life_this_turn(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    itself(view, source, affected, so_far) && view.history().life_lost(view.state().obj(source).controller) > 0
}

/// "... this creature has flying and vigilance" (CR 702.9, 702.20, layer 6).
fn flying_and_vigilance(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Flying).with(Keyword::Vigilance);
}

/// "When this creature dies, put its counters on target creature you control": as many of each kind as it had as it died,
/// read from its last existence on the battlefield (CR 603.10a, 608.2h); the counters are put on, not moved (the rulings),
/// every kind of them, -1/-1 counters too.
fn pass_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    // A lone target that is no longer legal stops the whole ability (CR 608.2b), so this always finds one.
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let counters = g.obj(item.object).counter_array();
    for kind in CounterKind::ALL {
        let n = counters[kind as usize];
        if n > 0 {
            g.add_counters(creature, kind, n);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(YOU_GAIN_LIFE, plus_one_counter_on_this,"Whenever you gain life, put a +1/+1 counter on this creature."),
        triggered(THIS_DIES, pass_counters, "When this creature dies, put its counters on target creature you control.")
            .targets(&[targets::CREATURE_YOU_CONTROL]),
    ],
    statics: &[static_ability(
        Layer::Ability,
        you_lost_life_this_turn,
        flying_and_vigilance,
        "As long as you've lost life this turn, this creature has flying and vigilance.",
    )],
    ..oracle::ESSENCE_CHANNELER
};
