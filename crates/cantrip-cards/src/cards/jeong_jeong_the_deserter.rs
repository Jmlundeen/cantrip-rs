use cantrip_api::*;
use cantrip_mechanics::{copy, delayed, effects, firebending};
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on Jeong Jeong. When you next cast a Lesson spell this turn, copy it and you may choose new targets for
/// the copy": the counter, then a delayed triggered ability, number 0 of the card's `delayed` list, that triggers once on the
/// next Lesson spell its controller casts this turn and ends with the turn if none is cast (CR 603.7b).
fn counter_and_watch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::plus_one_counter_on_this(g, item)?;
    delayed::next_this_turn(g, item, 0, item.object);
    Ok(())
}

/// "When you cast a Lesson spell" (CR 603.2, 601.2i, 205.3k): a spell cast, not one copied (CR 707.10), by this ability's
/// controller, with the subtype Lesson as it stands on the stack. The spell rides along as the trigger's subject.
fn you_cast_a_lesson_spell(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. } if *player == me.controller && q.chars(*spell).has_subtype(Subtype::Lesson) => {
            Fired::about(Target::Object(*spell))
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        firebending::trigger::<1>("Firebending 1 (Whenever this creature attacks, add {R}. This mana lasts until end of combat.)"),
        ability(
            cost("{3}"),
            counter_and_watch,
            "Exhaust \u{2014} {3}: Put a +1/+1 counter on this creature. When you next cast a Lesson spell this turn, copy it and you may choose new targets for the copy. (Activate each exhaust ability only once.)",
        )
        .exhaust(),
    ],
    delayed: &[triggered(
        when_asked(EventMask::SPELL_CAST, you_cast_a_lesson_spell),
        copy::copy_the_cast_spell,
        "When you next cast a Lesson spell this turn, copy it and you may choose new targets for the copy.",
    )],
    ..oracle::JEONG_JEONG_THE_DESERTER
};
