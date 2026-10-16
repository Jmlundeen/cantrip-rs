use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::ward_life;
use cantrip_oracle as oracle;

fn instant_or_sorcery(q: &dyn Query, spell: ObjectId) -> bool {
    let c = q.chars(spell);
    c.is(CardType::Instant) || c.is(CardType::Sorcery)
}

/// "Whenever you cast or copy an instant or sorcery spell" (CR 603.2): the two events share one ability. A copy of a
/// spell is not cast (CR 707.10a), and a copy of a permanent is no spell, so only a copy on the stack counts.
fn cast_or_copy(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    let you = q.controller(source);
    match event {
        Event::SpellCast { spell, player, .. } if *player == you && instant_or_sorcery(q, *spell) => Fired::PLAIN,
        Event::Copied { copy, player, .. }
            if *player == you
                && q.state().stack.iter().any(|i| i.kind == StackKind::Spell && i.object == *copy)
                && instant_or_sorcery(q, *copy) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

fn create_a_pest(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::PEST, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ward_life::<3>("Ward\u{2014}Pay 3 life. (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays 3 life.)"),
        triggered(when(EventMask(EventMask::SPELL_CAST.0 | EventMask::COPIED.0), cast_or_copy), create_a_pest, "Magecraft \u{2014} Whenever you cast or copy an instant or sorcery spell, create a 1/1 black and green Pest creature token with \"When this token dies, you gain 1 life.\""),
    ],
    ..oracle::SEDGEMOOR_WITCH
};

pub static EXTRA: &[&CardDef] = &[&tokens::PEST];
