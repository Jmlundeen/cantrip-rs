use cantrip_api::*;
use cantrip_mechanics::library::{Find, search_to_battlefield};
use cantrip_mechanics::{delayed, effects, lands};
use cantrip_oracle as oracle;

/// I: "Search your library for a basic land card, put it onto the battlefield tapped, then shuffle" (CR 701.23).
fn crescent_fang(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_battlefield(g, item.controller, item.object, Find::ONE, true, lands::is_basic_land_card)?;
    Ok(())
}

/// II: "When you next cast a creature spell this turn" (CR 603.7b): the chapter sets up a delayed triggered ability that
/// triggers once, on the first creature spell its controller casts (not copies, CR 707.10a) this turn, read as the spell
/// stands on the stack, and ends with the turn if none is cast. The spell rides along as the trigger's subject.
fn next_creature_spell_you_cast(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. } if *player == me.controller && q.chars(*spell).is(CardType::Creature) => {
            Fired::about(Target::Object(*spell))
        }
        _ => None,
    }
}

fn heavenward_howl(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    delayed::next_this_turn(g, item, 0, item.object);
    Ok(())
}

/// The delayed trigger's effect: "that creature enters with an additional +1/+1 counter on it" (CR 614.1c, 122.6). A
/// replacement effect about the spell for the turn, applied when it enters the battlefield from the stack (CR 614.12).
fn give_the_spell_a_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(spell) = item.subject {
        g.add_effect(item.object, spell, EffectKind::Replacement(0), Duration::EndOfTurn);
    }
    Ok(())
}

fn spell_enters(_: &dyn Query, _: &Asker, affected: ObjectId, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::ZoneChange(z) if z.object == affected && z.from.is(ZoneKind::Stack) && z.to.is(ZoneKind::Battlefield))
}

fn add_a_counter(_: &dyn Query, _: &Asker, _: ObjectId, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.counters[CounterKind::PlusOne as usize] += 1;
    }
}

/// III: "Draw a card if you control the creature with the greatest power or tied for the greatest power": judged as the
/// chapter resolves, among all creatures on the battlefield, Fenrir itself included while it is still there.
fn ecliptic_growl(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<(ObjectId, i16)> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.chars(id).is(CardType::Creature))
        .map(|id| (id, g.chars(id).power.unwrap_or(0)))
        .collect();
    let greatest = creatures.iter().map(|&(_, power)| power).max();
    if creatures.iter().any(|&(id, power)| Some(power) == greatest && g.controller(id) == item.controller) {
        return effects::draw(g, item, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        chapter(1, crescent_fang, "I \u{2014} Crescent Fang \u{2014} Search your library for a basic land card, put it onto the battlefield tapped, then shuffle."),
        chapter(2, heavenward_howl, "II \u{2014} Heavenward Howl \u{2014} When you next cast a creature spell this turn, that creature enters with an additional +1/+1 counter on it."),
        chapter(3, ecliptic_growl, "III \u{2014} Ecliptic Growl \u{2014} Draw a card if you control the creature with the greatest power or tied for the greatest power."),
    ],
    delayed: &[triggered(
        when_asked(EventMask::SPELL_CAST, next_creature_spell_you_cast),
        give_the_spell_a_counter,
        "When you next cast a creature spell this turn, that creature enters with an additional +1/+1 counter on it.",
    )],
    granted_replacements: &[granted_replacement(ReplacementKind::ZoneChange, spell_enters, add_a_counter, "enters with an additional +1/+1 counter")],
    ..oracle::SUMMON_FENRIR
};
