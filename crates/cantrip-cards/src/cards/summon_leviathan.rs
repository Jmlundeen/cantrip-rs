use cantrip_api::*;
use cantrip_mechanics::{delayed, triggers};
use cantrip_oracle as oracle;

/// Whether `id` is a Kraken, Leviathan, Merfolk, Octopus, or Serpent, the subtypes read through the words of `source`'s
/// rules text (CR 612) so a text-changing effect changes them.
fn is_kraken_leviathan_merfolk_octopus_or_serpent(q: &dyn Query, source: ObjectId, id: ObjectId) -> bool {
    let words = q.words(source);
    let chars = q.chars(id);
    [Subtype::Kraken, Subtype::Leviathan, Subtype::Merfolk, Subtype::Octopus, Subtype::Serpent]
        .into_iter()
        .any(|subtype| chars.has_subtype(words.subtype(subtype)))
}

/// "I — Return each creature that isn't a Kraken, Leviathan, Merfolk, Octopus, or Serpent to its owner's hand."
/// (CR 400.3): every such creature on the battlefield, returned at once.
fn return_the_others(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> = {
        let q: &dyn Query = g;
        q.state()
            .battlefield()
            .iter()
            .copied()
            .filter(|&id| q.chars(id).is(CardType::Creature) && !is_kraken_leviathan_merfolk_octopus_or_serpent(q, item.object, id))
            .collect()
    };
    g.return_to_hand(&creatures);
    Ok(())
}

/// The delayed ability's event: "whenever a Kraken, Leviathan, Merfolk, Octopus, or Serpent attacks", whoever controls it.
fn such_a_creature_attacks(q: &dyn Query, asker: &Asker, event: &Event) -> Option<Fired> {
    let Event::Attacks { attacker, .. } = event else { return None };
    if is_kraken_leviathan_merfolk_octopus_or_serpent(q, asker.source, *attacker) {
        return Fired::about(Target::Object(*attacker));
    }
    None
}

/// "Draw a card." (CR 121.1), for the delayed ability.
fn draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

/// "II, III — Until end of turn, whenever a Kraken, Leviathan, Merfolk, Octopus, or Serpent attacks, draw a card." The
/// delayed triggered ability lasts the turn (CR 603.7b), controlled by the chapter's controller (CR 603.7d-e).
fn until_end_of_turn_draw_when_they_attack(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    delayed::this_turn(g, item, item.object);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        chapter(
            1,
            return_the_others,
            "I — Return each creature that isn't a Kraken, Leviathan, Merfolk, Octopus, or Serpent to its owner's hand.",
        ),
        chapter(
            2,
            until_end_of_turn_draw_when_they_attack,
            "II, III — Until end of turn, whenever a Kraken, Leviathan, Merfolk, Octopus, or Serpent attacks, draw a card.",
        ),
        chapter(
            3,
            until_end_of_turn_draw_when_they_attack,
            "II, III — Until end of turn, whenever a Kraken, Leviathan, Merfolk, Octopus, or Serpent attacks, draw a card.",
        ),
        triggers::ward::<2>("Ward {2}"),
    ],
    delayed: &[triggered(
        when_asked(EventMask::ATTACKS, such_a_creature_attacks),
        draw,
        "Until end of turn, whenever a Kraken, Leviathan, Merfolk, Octopus, or Serpent attacks, draw a card.",
    )],
    ..oracle::SUMMON_LEVIATHAN
};
