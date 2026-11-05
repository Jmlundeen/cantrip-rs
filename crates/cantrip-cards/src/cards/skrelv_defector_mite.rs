use cantrip_api::*;
use cantrip_mechanics::{choose_color, targets};
use cantrip_oracle as oracle;

/// "It can't be blocked by creatures of that color" (CR 509.1b): the restriction on the creature `on`, as an attacker, that a
/// creature of the color `COLOR` (a `Color` as `u8`) may not block it, read from the blocker's colors as blockers are declared.
fn not_blocked_by<const COLOR: u8>(q: &dyn Query, on: ObjectId, blocker: ObjectId, attacker: ObjectId) -> bool {
    on != attacker || !q.chars(blocker).colors.has(Color::ALL[COLOR as usize])
}

const fn rule<const COLOR: u8>(text: &'static str) -> LastingBlockRule {
    LastingBlockRule { text, allows: not_blocked_by::<COLOR> }
}

/// One rule for each color, in `Color::ALL` order: the effect's `index` is the color chosen.
static BLOCK_RULES: [LastingBlockRule; 5] = [
    rule::<{ Color::White as u8 }>("Can't be blocked by white creatures"),
    rule::<{ Color::Blue as u8 }>("Can't be blocked by blue creatures"),
    rule::<{ Color::Black as u8 }>("Can't be blocked by black creatures"),
    rule::<{ Color::Red as u8 }>("Can't be blocked by red creatures"),
    rule::<{ Color::Green as u8 }>("Can't be blocked by green creatures"),
];

/// "Choose a color. Another target creature you control gains toxic 1 and hexproof from that color until end of turn. It can't
/// be blocked by creatures of that color this turn." The color is chosen as the ability resolves (CR 608.2d), before anything
/// is done; a target that has left the battlefield is left alone (CR 608.2b). The gains and the restriction are continuous
/// effects of the resolving ability on the target (CR 611.2a), the hexproof for that color for good (CR 702.11d).
fn share_the_toxin(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let color = choose_color(g, item.controller, item.object)?;
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    instruction(g, |g| {
        g.add_effect(item.object, creature, EffectKind::Toxic(1), Duration::EndOfTurn);
        g.add_effect(item.object, creature, EffectKind::HexproofFrom(HexproofFrom::color(color)), Duration::EndOfTurn);
    });
    g.end_sentence();
    g.add_effect(item.object, creature, EffectKind::BlockRestriction { card: item.ability.card, index: color as u8 }, Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    keywords: Keywords::of(&[Keyword::CantBlock]),
    abilities: &[ability(
        cost("{W/P}").and(TAP),
        share_the_toxin,
        "{W/P}, {T}: Choose a color. Another target creature you control gains toxic 1 and hexproof from that color until end of turn. It can't be blocked by creatures of that color this turn.",
    )
    .targets(&[targets::OTHER_CREATURE_YOU_CONTROL])],
    lasting_block_rules: &BLOCK_RULES,
    ..oracle::SKRELV_DEFECTOR_MITE
};
