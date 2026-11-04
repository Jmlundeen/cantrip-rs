use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Whenever you cast a creature spell with mana value equal to Jackal's power" (CR 603.2, 601.2i, 202.3): a spell cast, not one
/// copied (CR 707.10), by this ability's controller, read as it stands on the stack (X counted, CR 202.3e) against Jackal's power
/// as the spell is cast. The spell rides along as the trigger's subject ("that spell"). Jackal's own casting never triggers it: he
/// is not on the battlefield.
fn you_cast_a_creature_spell_with_mana_value_equal_to_his_power(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. }
            if *player == q.controller(source) && q.chars(*spell).is(CardType::Creature) && q.chars(source).power.is_some_and(|power| u32::try_from(power) == Ok(q.mana_value(*spell))) =>
        {
            Fired::about(Target::Object(*spell))
        }
        _ => None,
    }
}

/// "Copy that spell, except the copy isn't legendary. Then put a +1/+1 counter on Jackal" (CR 707.10, 707.9b, 122.1): the copy is
/// not cast and keeps the spell's targets, a mutating spell's among them, and a copy of a permanent spell becomes a token as it
/// resolves (CR 707.10f). It is made from the spell's last-known information if the spell has left the stack (CR 608.2h). The
/// counter goes on Jackal whatever became of the copy.
fn copy_that_spell_then_grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(spell) = item.subject else { return Ok(()) };
    let values = g.copiable_values(spell).nonlegendary();
    // The copy keeps the original's targets - a mutating creature spell has one (CR 702.140a) - because Jackal doesn't say you
    // may choose new ones (CR 707.10c).
    g.copy_spell_as(spell, item.controller, false, values)?;
    effects::plus_one_counter_on_this(g, item)
}

/// Trample is a core keyword.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when(EventMask::SPELL_CAST, you_cast_a_creature_spell_with_mana_value_equal_to_his_power),
        copy_that_spell_then_grow,
        "Whenever you cast a creature spell with mana value equal to Jackal's power, copy that spell, except the copy isn't legendary. Then put a +1/+1 counter on Jackal.",
    )],
    ..oracle::JACKAL_GENIUS_GENETICIST
};
