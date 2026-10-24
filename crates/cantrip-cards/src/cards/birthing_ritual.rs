use cantrip_api::*;
use cantrip_mechanics::{choose, effects, library, top_of_library, triggers};
use cantrip_oracle as oracle;

const SACRIFICE: Question = Question::new("SacrificeCreature", "sacrifice a creature?", "If you do, you may put a creature card with mana value X or less from among the cards looked at onto the battlefield", "Sacrifice a creature", "Don't sacrifice");

fn controls_creature(q: &dyn Query, asker: &Asker) -> bool {
    q.permanents_of(asker.controller).into_iter().any(|id| q.chars(id).is(CardType::Creature))
}

/// "Look at the top seven ... Then you may sacrifice a creature. If you do, you may put a creature card with mana value
/// X or less ... where X is 1 plus the sacrificed creature's mana value. Put the rest on the bottom ... in a random order."
/// The sacrificed object's last-known mana value is used (CR 608.2h); the looked-at cards remain in the library unless
/// one is put onto the battlefield (CR 401.2, 401.4).
fn ritual(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let looked = top_of_library(g, you, 7);
    g.look_at(you, &looked);
    let mut maximum = None;
    // Only a creature that can be sacrificed is one to sacrifice (CR 101.2): with none, nothing is asked.
    let creatures = g.permanents_of(you).into_iter().filter(|&id| g.chars(id).is(CardType::Creature) && g.can_sacrifice(id)).collect::<Vec<_>>();
    if !creatures.is_empty()
        && effects::ask_may(g, you, item.object, Purpose::Ask(&SACRIFICE))?
        && let Some(creature) = choose::card(g, you, item.object, Purpose::Sacrifice, creatures)?
    {
        let mana_value = g.mana_value(creature);
        let sacrificed = g.sacrifice(&[creature]);
        if !sacrificed.is_empty() {
            maximum = Some(1 + mana_value);
        }
    }
    if let Some(maximum) = maximum {
        let eligible = looked.iter().copied().filter(|&card| g.chars(card).is(CardType::Creature) && g.mana_value(card) <= maximum).collect();
        if let Some(card) = choose::card_from(g, you, item.object, Purpose::PutOntoBattlefield, looked.clone(), eligible)? {
            g.put_all_onto_battlefield(&[(card, you)])?;
        }
    }
    library::rest_to_bottom_in_random_order(g, you, &looked);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::YOUR_END_STEP, ritual,
        "At the beginning of your end step, if you control a creature, look at the top seven cards of your library. Then you may sacrifice a creature. If you do, you may put a creature card with mana value X or less from among those cards onto the battlefield, where X is 1 plus the sacrificed creature's mana value. Put the rest on the bottom of your library in a random order.")
        .condition(controls_creature)],
    ..oracle::BIRTHING_RITUAL
};
