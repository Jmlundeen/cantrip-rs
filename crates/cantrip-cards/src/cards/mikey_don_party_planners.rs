use cantrip_api::*;
use cantrip_mechanics::triggers::ward;
use cantrip_oracle as oracle;

/// "Cast Mutant, Ninja, or Turtle spells from the top of your library" (CR 601.3): the card on top as the spell it would be, its
/// creature types read through the words of this permanent's text (CR 612).
fn mutant_ninja_or_turtle_spell(q: &dyn Query, asker: &Asker, card: ObjectId) -> bool {
    let words = q.words(asker.source);
    let c = q.chars(card);
    c.has_subtype(words.subtype(Subtype::Mutant)) || c.has_subtype(words.subtype(Subtype::Ninja)) || c.has_subtype(words.subtype(Subtype::Turtle))
}

/// "If you cast a creature spell this way, that creature enters with an additional +1/+1 counter on it" (CR 614.1c, 122.6):
/// a replacement effect about the spell this permanent's rule let its controller cast from the top of the library
/// (`PlayRule::LibraryTopSpells`' rider), made as it is cast (CR 601.2a) and so still applying if Mikey & Don has left before
/// it resolves. It applies as that spell, a creature spell, enters the battlefield from the stack.
fn creature_spell_enters(q: &dyn Query, _: &Asker, spell: ObjectId, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::ZoneChange(z)
        if z.object == spell && z.from.is(ZoneKind::Stack) && z.to.is(ZoneKind::Battlefield) && q.chars(spell).is(CardType::Creature))
}

fn add_a_counter(_: &dyn Query, _: &Asker, _: ObjectId, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.counters[CounterKind::PlusOne as usize] += 1;
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[ward::<2>("Ward {2} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {2}.)")],
    // "You may look at the top card of your library any time" (CR 401.2), and "you may play lands and cast Mutant, Ninja, or Turtle
    // spells from the top of your library" (CR 305.1, 601.3).
    play_rules: &[PlayRule::LookAtLibraryTop, PlayRule::LibraryTopLands, PlayRule::LibraryTopSpells(mutant_ninja_or_turtle_spell, Some(0))],
    granted_replacements: &[granted_replacement(
        ReplacementKind::ZoneChange,
        creature_spell_enters,
        add_a_counter,
        "enters with an additional +1/+1 counter on it",
    )],
    ..oracle::MIKEY_DON_PARTY_PLANNERS
};
