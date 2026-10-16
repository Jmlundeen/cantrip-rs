use cantrip_api::*;
use cantrip_mechanics::evoke;
use cantrip_mechanics::{discard_chosen, exile_target};
use cantrip_oracle as oracle;

/// "If {X}{X} was spent to cast it": at least two mana of that color among everything spent on it, generic included.
/// A permanent that was not cast had nothing spent.
fn spent_two(q: &dyn Query, source: ObjectId, mana: Mana) -> bool {
    q.obj(source).spent.get(mana) >= 2
}

fn entered(source: ObjectId, event: &Event) -> bool {
    matches!(event, Event::ZoneChange { new, to, .. } if *new == source && to.is(ZoneKind::Battlefield))
}

fn green_entered(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    if entered(source, event) && spent_two(q, source, Mana::G) { Fired::PLAIN } else { None }
}

fn blue_entered(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    if entered(source, event) && spent_two(q, source, Mana::U) { Fired::PLAIN } else { None }
}

fn green_spent(q: &dyn Query, asker: &Asker) -> bool {
    spent_two(q, asker.source, Mana::G)
}

fn blue_spent(q: &dyn Query, asker: &Asker) -> bool {
    spent_two(q, asker.source, Mana::U)
}

fn artifact_or_enchantment_an_opponent_controls(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    match t {
        Target::Object(id) if is_live(q, id) && q.is_on_battlefield(id) => {
            let c = q.chars(id);
            q.controller(id) != asker.controller && (c.is(CardType::Artifact) || c.is(CardType::Enchantment))
        }
        _ => false,
    }
}

fn exile_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    Ok(())
}

fn draw_then_discard(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 2);
    discard_chosen(g, item.controller, item.object, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    ways: &[evoke::way(Cost::mana(ManaCost {
            generic: 0,
            symbols: [0; 6],
            hybrid: [Some((Mana::G, Mana::U)), Some((Mana::G, Mana::U)), None, None, None, None],
            phyrexian: [0; 5],
            x: 0,
        }), "Evoke {G/U}{G/U}")],
    abilities: &[
        evoke::sacrifice_trigger("When Wistfulness enters, if its evoke cost was paid, its controller sacrifices it."),
        triggered(when(EventMask::ZONE_CHANGE, green_entered), exile_it, "When this creature enters, if {G}{G} was spent to cast it, exile target artifact or enchantment an opponent controls.")
            .targets(&[target(artifact_or_enchantment_an_opponent_controls)])
            .condition(green_spent),
        triggered(when(EventMask::ZONE_CHANGE, blue_entered), draw_then_discard, "When this creature enters, if {U}{U} was spent to cast it, draw two cards, then discard a card.")
            .condition(blue_spent),
    ],
    ..oracle::WISTFULNESS
};
