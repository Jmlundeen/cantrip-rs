use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

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

pub static CARD: CardDef = CardDef {
    enters: Some(effects::enters_with_x_counters::<{ CounterKind::Charge as usize }>),
    abilities: &[triggered(
        when(EventMask::SPELL_CAST, spell_of_equal_mana_value),
        effects::counter_subject_spell,
        "Whenever a player casts a spell with mana value equal to the number of charge counters on this artifact, counter that spell.",
    )],
    ..oracle::CHALICE_OF_THE_VOID
};
