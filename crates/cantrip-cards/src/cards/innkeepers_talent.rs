use cantrip_api::*;
use cantrip_mechanics::replacements::{double_counters_applies, double_counters_apply};
use cantrip_mechanics::{class, statics, targets, triggers::ward};
use cantrip_oracle as oracle;

fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    Ok(())
}

/// Level 2: "Permanents you control with counters on them have ward {1}": from level 2 on, each permanent its controller
/// has that has a counter on it, read afresh on every pass (CR 613.1f). Each Talent gives its own ward, so two give two,
/// each triggering separately (CR 113.2c, 702.21a; `statics::grant_delayed_ability`).
fn has_counters(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    let s = view.state();
    s.obj(source).class_level >= 2
        && s.obj(affected).controller == s.obj(source).controller
        && s.obj(affected).has_counters()
}

/// Level 3: "If you would put one or more counters on a permanent or player, put twice that many of each of those kinds
/// of counters on that permanent or player instead" (CR 122.6, 614.1a).
fn doubles(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    q.obj(asker.source).class_level >= 3 && double_counters_applies(proposal, asker)
}

fn double(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    double_counters_apply(proposal);
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(cantrip_mechanics::triggers::BEGINNING_OF_YOUR_COMBAT, counter, "At the beginning of combat on your turn, put a +1/+1 counter on target creature you control.")
            .targets(&[targets::CREATURE_YOU_CONTROL]),
        class::level_two("{G}", "{G}: Level 2"),
        class::level_three("{3}{G}", "{3}{G}: Level 3"),
    ],
    delayed: &[ward::<1>("Ward {1} (Whenever this permanent becomes the target of a spell or ability an opponent controls, counter it unless that player pays {1}.)")],
    statics: &[static_ability(Layer::Ability, has_counters, statics::grant_delayed_ability, "Permanents you control with counters on them have ward {1}.")],
    replacements: &[replacement(ReplacementKind::Counters, doubles, double, "puts twice that many counters")],
    ..oracle::INNKEEPERS_TALENT
};
