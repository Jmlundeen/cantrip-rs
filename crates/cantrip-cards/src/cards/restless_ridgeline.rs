use cantrip_api::*;
use cantrip_mechanics::{lands, mana, targets, triggers};
use cantrip_oracle as oracle;

/// "{2}{R}{G}: This land becomes a 3/4 red and green Dinosaur creature until end of turn. It's still a land." (CR 611.2a).
fn land_becomes_3_4_red(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let colors = ColorSet(ColorSet::of(Color::Red).0 | ColorSet::of(Color::Green).0);
    lands::becomes_creature_until_end_of_turn(g, item, (3, 4), colors, Subtype::Dinosaur, Keywords::NONE);
    Ok(())
}

/// "Another target attacking creature" (CR 115.1, 506.4): a creature attacking now, other than this land.
fn another_attacking_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    t != Target::Object(asker.source) && targets::attacking_creature_filter(q, asker, t)
}

/// "Whenever this land attacks, another target attacking creature gets +2/+0 until end of turn. Untap that creature."
/// (CR 608.2, 611.2a, 701.26b): one instruction ends where the next sentence begins (CR 608.2c).
fn pump_and_untap(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    instruction(g, |g| g.add_effect(item.object, creature, EffectKind::Pump(2, 0), Duration::EndOfTurn));
    g.end_sentence();
    g.untap(creature);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_one_of(&[Color::Red, Color::Green]), "{T}: Add {R} or {G}."),
        ability(
            cost("{2}{R}{G}"),
            land_becomes_3_4_red,
            "{2}{R}{G}: This land becomes a 3/4 red and green Dinosaur creature until end of turn. It's still a land.",
        ),
        triggered(
            triggers::THIS_ATTACKS,
            pump_and_untap,
            "Whenever this land attacks, another target attacking creature gets +2/+0 until end of turn. Untap that creature.",
        )
        .targets(&[target(another_attacking_creature)]),
    ],
    enters: Some(mana::enters_tapped),
    ..oracle::RESTLESS_RIDGELINE
};
