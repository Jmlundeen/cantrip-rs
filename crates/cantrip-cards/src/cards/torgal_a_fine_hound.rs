use cantrip_api::*;
use cantrip_mechanics::next_creature_counter::{give_the_spell_a_counter, spell_enters};
use cantrip_oracle as oracle;

/// Whether `spell` is a Human creature spell, as it was cast (CR 601.2i): "Human" is a word of Torgal's text (CR 612).
fn human_creature_spell(q: &dyn Query, source: ObjectId, chars: &Chars) -> bool {
    chars.is(CardType::Creature) && chars.has_subtype(q.words(source).subtype(Subtype::Human))
}

/// "Whenever you cast your first Human creature spell each turn" (CR 603.2): a Human creature spell its controller casts, and no
/// other Human creature spell they cast earlier this turn, each judged as it was cast. The spell rides along as the trigger's
/// subject.
fn first_human_creature_spell_you_cast(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let Event::SpellCast { spell, player, .. } = event else { return None };
    if *player != me.controller || !human_creature_spell(q, me.source, &q.chars(*spell)) {
        return None;
    }
    if q.history().casts_by(*player).any(|c| c.spell != *spell && human_creature_spell(q, me.source, &c.chars(q))) {
        return None;
    }
    Fired::about(Target::Object(*spell))
}

/// One +1/+1 counter for each Dog and/or Wolf its controller controls as it enters, Torgal himself a Wolf among them; a Dog
/// Wolf counts once (CR 122.6, 614.1c).
fn add_a_counter_for_each_dog_or_wolf(q: &dyn Query, asker: &Asker, _: ObjectId, proposal: &mut Proposal) {
    let words = q.words(asker.source);
    let (dog, wolf) = (words.subtype(Subtype::Dog), words.subtype(Subtype::Wolf));
    let count = q.permanents_of(asker.controller).into_iter().filter(|&id| q.chars(id).has_subtype(dog) || q.chars(id).has_subtype(wolf)).count();
    if let Proposal::ZoneChange(z) = proposal {
        z.counters[CounterKind::PlusOne as usize] += count as i32;
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::SPELL_CAST, first_human_creature_spell_you_cast),
            give_the_spell_a_counter,
            "Whenever you cast your first Human creature spell each turn, that creature enters with an additional +1/+1 counter on it for each Dog and/or Wolf you control.",
        ),
        mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color."),
    ],
    granted_replacements: &[granted_replacement(
        ReplacementKind::ZoneChange,
        spell_enters,
        add_a_counter_for_each_dog_or_wolf,
        "enters with an additional +1/+1 counter",
    )
    .text("that creature enters with an additional +1/+1 counter on it for each Dog and/or Wolf you control.")],
    ..oracle::TORGAL_A_FINE_HOUND
};
