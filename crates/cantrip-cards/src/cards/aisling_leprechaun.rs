use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Whenever this creature blocks or becomes blocked by a creature" (CR 603.2): the other creature in the block is the subject.
fn blocks_or_becomes_blocked(q: &dyn Query, asker: &Asker, event: &Event) -> Option<Fired> {
    let other = triggers::other_creature_in_block_with_this(q, asker.source, event)?;
    Fired::about(Target::Object(other))
}

/// "... that creature becomes green." (CR 608.2, 611.2a): the creature the trigger is about, green with no end to it (CR 105.3).
fn creature_becomes_green(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(other) = item.subject.object() else { return Ok(()) };
    let kind = EffectKind::SetColors(g.words(item.object).colors(ColorSet::of(Color::Green)));
    instruction(g, |g| g.add_effect(item.object, other, kind, Duration::Indefinite));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::BLOCKS, blocks_or_becomes_blocked),
        creature_becomes_green,
        "Whenever this creature blocks or becomes blocked by a creature, that creature becomes green.",
    )],
    ..oracle::AISLING_LEPRECHAUN
};
