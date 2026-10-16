use cantrip_api::*;
use cantrip_mechanics::library::object_has_name;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "When this land enters, choose a land card name." The name is chosen as the trigger resolves, of a land card
/// (CR 201.4, 201.4a), and remembered by this land as the object it is (CR 607.2d): if it has left the battlefield by
/// then, the choice is made and nothing keeps it, and a Hamlet that comes back is a new object with a trigger of its own
/// (CR 400.7). An unscripted player names the land an opponent controls most of among those with an ability that is not
/// a mana ability, the ones this stops.
fn choose_land_name(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let mut seen: Vec<(CardId, usize)> = Vec::new();
    for opponent in g.state().opponents(you).collect::<Vec<_>>() {
        for land in g.permanents_of(opponent) {
            let c = g.chars(land);
            let stopped = g.card(c.card).abilities.iter().any(|a| matches!(a, AbilityDef::Activated(d) if !d.is_mana_ability()));
            if c.is(CardType::Land) && stopped {
                match seen.iter_mut().find(|(card, _)| *card == c.card) {
                    Some((_, n)) => *n += 1,
                    None => seen.push((c.card, 1)),
                }
            }
        }
    }
    let suggestion = seen.iter().max_by_key(|(_, n)| *n).map(|(card, _)| *card);
    let ask = RequestKind::ChooseName { source: item.object, looked_at: Vec::new(), suggestion, kind: NameKind::Land };
    if let Answer::Name(Some(name)) = g.ask(you, ask)? {
        g.remember_chosen_name(item.object, name);
    }
    Ok(())
}

/// "Activated abilities of sources with the chosen name can't be activated unless they're mana abilities." (CR 602.5,
/// 605.1a.) A source is anything with the name, in any zone it has the ability in: a land on the battlefield, a land card's
/// channel or cycling in a hand. It binds every player, the Hamlet's controller too; with no name chosen it stops nothing.
fn may_activate(q: &dyn Query, asker: &Asker, source: ObjectId, def: &ActivatedDef) -> bool {
    def.is_mana_ability() || !q.obj(asker.source).marks.chosen_card.is_some_and(|name| object_has_name(q, source, name))
}

/// "Lands with the chosen name have "{T}: Add {C}."" (CR 613.1f): the land, a permanent (CR 109.2), has the Hamlet's own
/// delayed ability 0 for as long as it has the name and the Hamlet is on the battlefield.
fn named_land(view: &dyn LayerQuery, source: ObjectId, _: ObjectId, chars: &Chars) -> bool {
    chars.is(CardType::Land)
        && view.state().obj(source).marks.chosen_card.is_some_and(|name| chars.name == view.cards().get(name).name)
}

fn grant_mana_ability(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let o = view.state().obj(source);
    let card = o.copy.map(|c| c.card).unwrap_or(o.card);
    if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
        chars.abilities[slot] = Some(AbilityRef::delayed(card, 0));
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        triggered(THIS_ENTERS, choose_land_name, "When this land enters, choose a land card name."),
    ],
    delayed: &[mana_ability(TAP, add(Mana::C), "{T}: Add {C}.")],
    statics: &[static_ability(Layer::Ability, named_land, grant_mana_ability, "Lands with the chosen name have \"{T}: Add {C}.\"")],
    activation_rules: &[may_activate],
    ..oracle::PETRIFIED_HAMLET
};
