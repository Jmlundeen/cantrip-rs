use cantrip_api::select::part;
use cantrip_api::*;
use cantrip_mechanics::{library, same_name, select};
use cantrip_oracle as oracle;

const SEARCH_FOR_AN_AURA: Question = Question::new(
    "SearchForAura",
    "search your library for an Aura card?",
    "Its mana value is no greater than that of the Aura that entered, and its name is different from each Aura you control; it enters attached to Light-Paws",
    "Search",
    "Don't search",
);

/// "Whenever an Aura you control enters, if you cast it" (CR 603.2, 603.4, 603.6a): an Aura permanent under its controller's
/// control that entered as the spell that resolved, which the history says its controller cast. The Aura is the trigger's subject.
fn aura_you_control_enters_if_you_cast_it(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let Event::ZoneChange { old, new, to, reason: ZoneReason::Resolve, .. } = event else { return None };
    let aura = q.words(me.source).subtype(Subtype::Aura);
    if to.is(ZoneKind::Battlefield)
        && q.controller(*new) == me.controller
        && q.chars(*new).has_subtype(aura)
        && q.history().cast_of(*old).is_some_and(|cast| cast.player == me.controller)
    {
        return Fired::about(Target::Object(*new));
    }
    None
}

fn aura_card(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    let aura = q.words(choice.source).subtype(Subtype::Aura);
    Verdict::each(choice, |card| q.chars(card).has_subtype(aura))
}

fn with_a_different_name_than_each_aura_you_control(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    let aura = q.words(choice.source).subtype(Subtype::Aura);
    let auras: Vec<ObjectId> = q.permanents_of(choice.chooser).into_iter().filter(|&id| q.chars(id).has_subtype(aura)).collect();
    Verdict::each(choice, |card| !auras.iter().any(|&on_battlefield| same_name(q, card, on_battlefield)))
}

/// "An Aura card with a different name than each Aura you control" (CR 701.23, 201.2), one at most.
const AURA_CARD_WITH_A_DIFFERENT_NAME: Selection = Selection::of(
    select::up_to(1),
    &[
        part(&(aura_card as SelectFn), 0, ""),
        part(&(with_a_different_name_than_each_aura_you_control as SelectFn), 0, "with a different name than each Aura you control"),
    ],
)
.called("Aura");

/// Whether `player` cast the spell `aura` entered as, by the history (CR 603.4: the intervening "if you cast it" is judged again as the
/// ability resolves, by its controller - a copy of the ability that an opponent controls does nothing).
fn cast_by(q: &dyn Query, aura: ObjectId, player: PlayerId) -> bool {
    let spell = q.history().all().iter().rev().find_map(|r| match r.event {
        Event::ZoneChange { old, new, reason: ZoneReason::Resolve, .. } if new == aura => Some(old),
        _ => None,
    });
    spell.and_then(|spell| q.history().cast_of(spell)).is_some_and(|cast| cast.player == player)
}

/// "You may search your library for an Aura card ..., put that card onto the battlefield attached to Light-Paws, then shuffle."
/// (CR 701.23, 303.4f, 701.24a).
fn search_for_an_aura(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(entered) = item.subject.object() else { return Ok(()) };
    if !cast_by(g, entered, item.controller) {
        return Ok(());
    }
    if !library::ask_may_search(g, item.controller, item.object, &SEARCH_FOR_AN_AURA)? {
        return Ok(());
    }
    let rule = AURA_CARD_WITH_A_DIFFERENT_NAME.with(select::mana_value_at_most(g.mana_value(entered)));
    let found = library::search(g, item.controller, item.controller, item.object, SearchDestination::Battlefield, rule)?;
    for card in found {
        g.put_onto_battlefield_attached(card, item.controller, item.object)?;
    }
    g.shuffle(item.controller);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, aura_you_control_enters_if_you_cast_it),
        search_for_an_aura,
        "Whenever an Aura you control enters, if you cast it, you may search your library for an Aura card with mana value less than or equal to that Aura and with a different name than each Aura you control, put that card onto the battlefield attached to Light-Paws, then shuffle.",
    )],
    ..oracle::LIGHT_PAWS_EMPERORS_VOICE
};
