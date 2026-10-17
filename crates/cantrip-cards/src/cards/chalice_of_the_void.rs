use cantrip_api::*;
use cantrip_oracle as oracle;

/// "This artifact enters with X charge counters on it" (CR 614.1c): the X it was cast with; put onto the battlefield
/// without being cast, X is 0 (CR 107.3).
fn enters_with_x_charge_counters(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.counters[CounterKind::Charge as usize] += entering.x as i16;
    Ok(())
}

/// "Whenever a player casts a spell with mana value equal to the number of charge counters on this artifact": any
/// player's spell, its mana value read on the stack, X as announced (CR 202.3e), against the counters there are as it is
/// cast (its ruling: a later change of counters changes nothing). The spell rides along as the trigger's subject.
fn spell_of_equal_mana_value(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, .. } if q.mana_value(*spell) as i32 == q.obj(source).counters(CounterKind::Charge) as i32 => {
            Fired::about(Target::Object(*spell))
        }
        _ => None,
    }
}

/// "Counter that spell" (CR 701.6): the spell that was cast, still on the stack; one that has left it is gone (CR 400.7),
/// and one that can't be countered stays.
fn counter_that_spell(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(spell) = item.subject {
        g.counter(spell);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters_with_x_charge_counters),
    abilities: &[triggered(
        when(EventMask::SPELL_CAST, spell_of_equal_mana_value),
        counter_that_spell,
        "Whenever a player casts a spell with mana value equal to the number of charge counters on this artifact, counter that spell.",
    )],
    ..oracle::CHALICE_OF_THE_VOID
};
