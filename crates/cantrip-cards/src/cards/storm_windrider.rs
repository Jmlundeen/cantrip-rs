use cantrip_api::*;
use cantrip_mechanics::{effects::gain_until_end_of_turn, targets};
use cantrip_oracle as oracle;

/// "Creatures with flying can't attack you" (CR 508.1c): a creature with flying may not attack Storm's controller; their
/// planeswalkers and battles stay open to it. A creature already attacking stays so if it gains flying afterwards
/// (CR 506.4a).
fn flyers_cant_attack_you(q: &dyn Query, asker: &Asker, creature: ObjectId, at: Target) -> bool {
    at != Target::Player(asker.controller) || !q.chars(creature).has(Keyword::Flying)
}

/// "Creatures with flying can't ... block creatures you control" (CR 509.1b): of a possible block, a blocker with flying
/// may not block an attacker its own controller does not control and Storm's controller does.
fn flyers_cant_block_yours(q: &dyn Query, source: ObjectId, blocker: ObjectId, attacker: ObjectId) -> bool {
    if q.controller(attacker) != q.controller(source) {
        return true;
    }
    !q.chars(blocker).has(Keyword::Flying)
}

/// The creatures among what `spell` targeted as it was cast, on the battlefield now (`targets::objects_targeted_as_cast`).
fn creatures_targeted_as_cast(q: &dyn Query, spell: ObjectId) -> Vec<ObjectId> {
    targets::permanents_targeted_as_cast(q, spell).into_iter().filter(|&id| q.chars(id).is(CardType::Creature)).collect()
}

/// "Those creatures", fixed as the ability triggers (`TriggeredDef::members_of`, read back with `Query::event_objects`): a
/// permanent that becomes a creature later is not one of them, nor is one that stops being one left out.
fn those_creatures(q: &dyn Query, subject: Target) -> Vec<ObjectId> {
    subject.object().map(|spell| creatures_targeted_as_cast(q, spell)).unwrap_or_default()
}

/// "Whenever you cast a spell that targets one or more creatures": the cast is the event, the spell the ability is about
/// (`StackItem::subject`).
fn you_cast_a_spell_that_targets_creatures(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let Event::SpellCast { spell, player, .. } = event else { return None };
    if *player != me.controller || creatures_targeted_as_cast(q, *spell).is_empty() {
        return None;
    }
    Fired::about(Target::Object(*spell))
}

/// "Those creatures gain flying until end of turn" (CR 611.2a): the creatures the spell targeted as it was cast that were
/// creatures as the ability triggered, still on the battlefield as it resolves. A later change of the spell's targets
/// (CR 115.7) changes none of them, and the spell need not still be on the stack.
fn those_creatures_gain_flying(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> = g.event_objects(item).iter().copied().filter(|&id| targets::permanent(g, id)).collect();
    // One instruction for all of them (CR 611.2c).
    instruction(g, |g| {
        for id in creatures {
            gain_until_end_of_turn(g, item.object, id, Keyword::Flying);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::SPELL_CAST, you_cast_a_spell_that_targets_creatures),
        those_creatures_gain_flying,
        "Whenever you cast a spell that targets one or more creatures, those creatures gain flying until end of turn.",
    )
    .members_of(those_creatures)],
    attack_rules: &[flyers_cant_attack_you],
    block_rules: &[flyers_cant_block_yours],
    ..oracle::STORM_WINDRIDER
};
