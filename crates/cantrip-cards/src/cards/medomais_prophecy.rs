use cantrip_api::*;
use cantrip_mechanics::library::object_has_name;
use cantrip_mechanics::{delayed, effects, scry, top_of_library};
use cantrip_oracle as oracle;

/// I: "Scry 2." (CR 701.22.)
fn chapter_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    scry(g, item.controller, item.object, 2)
}

/// II: "Choose a card name." Any card name (CR 201.4), remembered by this Saga as the object it is (CR 607.2d) for its
/// chapter III to refer to; if the Saga has left the battlefield the name is still chosen and nothing keeps it. An
/// unscripted player names the cheapest spell in their hand, the one most likely cast on chapter III's turn (a split
/// card by one of its names, its left half's, CR 201.4b).
fn chapter_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let mut suggestion: Option<(u32, CardId)> = None;
    for card in g.state().zone(Zone::hand(item.controller)).to_vec() {
        let id = g.chars(card).card;
        let def = g.card(id);
        if !def.is(CardType::Land) && suggestion.is_none_or(|(value, _)| def.cost.mana_value() < value) {
            suggestion = Some((def.cost.mana_value(), def.split.map_or(id, |(left, _)| g.card_id(left))));
        }
    }
    let suggestion = suggestion.map(|(_, name)| name);
    let ask = RequestKind::ChooseName { source: item.object, looked_at: Vec::new(), suggestion, kind: NameKind::Any };
    if let Answer::Name(Some(name)) = g.ask(item.controller, ask)? {
        g.remember_chosen_name(g.linked(item), name);
    }
    Ok(())
}

/// III: "When you cast a spell with the chosen name for the first time this turn, draw two cards." A delayed triggered
/// ability for the rest of the turn (CR 603.7), controlled by the chapter's controller (CR 603.7e). Set up even with no name chosen,
/// it then can't trigger (the card's ruling).
fn chapter_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    delayed::this_turn(g, item, item.object);
    Ok(())
}

/// "When you cast a spell with the chosen name for the first time this turn": the name is the one chosen for this Saga
/// (CR 607.2d), read from it even once it has left (its last-known row); "you" is the delayed ability's controller
/// (CR 603.7e). The history this sees is the one before the cast (`docs/adding-cards.md`), so an earlier spell of that
/// name cast this turn - before or after the ability was set up - means this is not the first time.
fn first_named_spell_you_cast(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let Event::SpellCast { spell, player, .. } = *event else { return None };
    let you = me.controller;
    let name = q.state().chosen_card(q.linked_to(me))?;
    let earlier = q.history().casts_by(you).any(|c| c.spell != spell && object_has_name(q, c.spell, name));
    if player == you && object_has_name(q, spell, name) && !earlier { Fired::PLAIN } else { None }
}

/// IV: "Look at the top card of each player's library." Each player's in turn order; an empty library shows nothing.
fn chapter_four(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let tops: Vec<ObjectId> = g.state().apnap().flat_map(|p| top_of_library(g, p, 1)).collect();
    g.look_at(item.controller, &tops);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        chapter(1, chapter_one, "I \u{2014} Scry 2."),
        chapter(2, chapter_two, "II \u{2014} Choose a card name."),
        chapter(3, chapter_three, "III \u{2014} When you cast a spell with the chosen name for the first time this turn, draw two cards."),
        chapter(4, chapter_four, "IV \u{2014} Look at the top card of each player's library."),
    ],
    delayed: &[triggered(
        when_asked(EventMask::SPELL_CAST, first_named_spell_you_cast),
        |g, item| effects::draw(g, item, 2),
        "When you cast a spell with the chosen name for the first time this turn, draw two cards.",
    )],
    ..oracle::MEDOMAIS_PROPHECY
};
