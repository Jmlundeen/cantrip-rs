use cantrip_api::*;
use cantrip_mechanics::replacements::{double_counters_applies, double_counters_apply};
use cantrip_mechanics::{targets, triggers::ward};
use cantrip_oracle as oracle;

/// The level a Class is at (CR 716.4): a designation, never a counter.
fn level(q: &dyn Query, source: ObjectId) -> u8 {
    q.obj(source).class_level.max(1)
}

fn at_level_one(q: &dyn Query, asker: &Asker) -> bool {
    level(q, asker.source) == 1
}

fn at_level_two(q: &dyn Query, asker: &Asker) -> bool {
    level(q, asker.source) == 2
}

fn become_level_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.set_class_level(item.object, 2);
    Ok(())
}

fn become_level_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.set_class_level(item.object, 3);
    Ok(())
}

fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    Ok(())
}

/// Level 2: "Permanents you control with counters on them have ward {1}": from level 2 on, each permanent its controller
/// has that has a counter on it, read afresh on every pass (CR 613.1f).
fn has_counters(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    let s = view.state();
    s.obj(source).class_level >= 2
        && s.obj(affected).controller == s.obj(source).controller
        && s.obj(affected).counters.iter().any(|&n| n > 0)
}

fn grant_ward(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let o = view.state().obj(source);
    let card = o.copy.map(|c| c.card).unwrap_or(o.card);
    let ward = AbilityRef::delayed(card, 0);
    if !chars.abilities().any(|a| a == ward)
        && let Some(slot) = chars.abilities.iter().position(|a| a.is_none())
    {
        chars.abilities[slot] = Some(ward);
    }
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
        AbilityDef::Triggered(TriggeredDef {
            text: "At the beginning of combat on your turn, put a +1/+1 counter on target creature you control.",
            listens: EventMask::STEP_BEGAN,
            matches: cantrip_mechanics::triggers::beginning_of_your_combat,
            targets: &[targets::CREATURE_YOU_CONTROL],
            effect: counter,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{G}: Level 2",
            cost: Cost::mana(mana(0, 0, 0, 0, 0, 1)),
            timing: Timing::Sorcery,
            condition: Some(at_level_one),
            effect: become_level_two,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{3}{G}: Level 3",
            cost: Cost::mana(mana(3, 0, 0, 0, 0, 1)),
            timing: Timing::Sorcery,
            condition: Some(at_level_two),
            effect: become_level_three,
            ..ActivatedDef::BLANK
        }),
    ],
    delayed: &[ward::<1>("Ward {1} (Whenever this permanent becomes the target of a spell or ability an opponent controls, counter it unless that player pays {1}.)")],
    statics: &[StaticDef {
        text: "Permanents you control with counters on them have ward {1}.",
        layer: Layer::Ability,
        affects: has_counters,
        apply: grant_ward,
        ..StaticDef::BLANK
    }],
    replacements: &[ReplacementDef {
        hint: "puts twice that many counters",
        kind: ReplacementKind::Counters,
        applies: doubles,
        apply: double,
        ..ReplacementDef::BLANK
    }],
    ..oracle::INNKEEPERS_TALENT
};
