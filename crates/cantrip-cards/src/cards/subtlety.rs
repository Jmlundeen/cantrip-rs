use cantrip_api::*;
use cantrip_mechanics::{evoke, library, targets};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

fn blue_card(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).colors.has(q.words(asker.source).color(Color::Blue))
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
/// countered, and its owner (CR 108.3) chooses.
fn put_into_library(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(spell) = targets::object(g, item, 0) else { return Ok(()) };
    library::owner_puts_on_top_or_bottom(g, item.object, spell)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, put_into_library, "When this creature enters, choose up to one target creature spell or planeswalker spell. Its owner puts it on their choice of the top or bottom of their library.")
            .targets(&[up_to_one(creature_or_planeswalker_spell)]),
        evoke::sacrifice_trigger("When Subtlety enters, if its evoke cost was paid, its controller sacrifices it."),
    ],
    // Evoke: exile a blue card from your hand rather than pay this spell's mana cost (CR 118.9).
    ways: &[evoke::way(evoke::exile_from_hand(blue_card), "Evoke\u{2014}Exile a blue card from your hand")],
    ..oracle::SUBTLETY
};
