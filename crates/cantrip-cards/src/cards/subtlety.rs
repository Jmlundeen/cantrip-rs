use cantrip_api::*;
use cantrip_mechanics::effects::ask_may;
use cantrip_mechanics::{evoke, targets};
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

fn blue_card(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
    q.chars(id).colors.has(Color::Blue)
}

/// "Target creature spell or planeswalker spell" (CR 115.1): a spell on the stack with one of those types.
fn creature_or_planeswalker_spell(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::SPELL.filter)(q, asker, t)
        && matches!(t, Target::Object(id) if {
            let c = q.chars(id);
            c.is(CardType::Creature) || c.is(CardType::Planeswalker)
        })
}

/// "Its owner puts it on their choice of the top or bottom of their library": the spell leaves the stack without being
/// countered (CR 400.3), and its owner chooses; yes is the top, as the C# engine asks it.
fn put_into_library(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(spell) = targets::object(g, item, 0) else { return Ok(()) };
    let owner = g.obj(spell).owner;
    let top = ask_may(g, owner, item.object, Purpose::Optional)?;
    g.remove_spell(spell, if top { CounterTo::LibraryTop } else { CounterTo::LibraryBottom });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When this creature enters, choose up to one target creature spell or planeswalker spell. Its owner puts it on their choice of the top or bottom of their library.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            targets: &[TargetSpec { min: 0, max: 1, filter: creature_or_planeswalker_spell }],
            effect: put_into_library,
            ..TriggeredDef::BLANK
        }),
        evoke::sacrifice_trigger("When Subtlety enters, if its evoke cost was paid, its controller sacrifices it."),
    ],
    // Evoke: exile a blue card from your hand rather than pay this spell's mana cost (CR 118.9).
    ways: &[evoke::way(evoke::exile_from_hand(blue_card), "Evoke\u{2014}Exile a blue card from your hand")],
    ..oracle::SUBTLETY
};
