use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "If you've cast both a creature spell and a noncreature spell this turn" (CR 603.4): an intervening if, asked as the
/// beginning of combat starts and again as the ability resolves. The history keeps the spells cast this turn as they were
/// cast (CR 601.2i).
fn cast_a_creature_and_a_noncreature_spell(q: &dyn Query, asker: &Asker) -> bool {
    let mut creature = false;
    let mut noncreature = false;
    for cast in q.history().casts_by(asker.controller) {
        if cast.is(CardType::Creature) { creature = true } else { noncreature = true }
    }
    creature && noncreature
}

/// "Draw a card and put two +1/+1 counters on Eshki Dragonclaw" (CR 121.1, 122.1).
fn draw_and_grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    effects::two_plus_one_counters_on_this(g, item)
}

/// Vigilance and trample are core keywords read from its characteristics.
pub static CARD: CardDef = CardDef {
    abilities: &[
        triggers::ward::<1>("Ward {1} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {1}.)"),
        triggered(
            triggers::BEGINNING_OF_YOUR_COMBAT,
            draw_and_grow,
            "At the beginning of combat on your turn, if you've cast both a creature spell and a noncreature spell this turn, draw a card and put two +1/+1 counters on this creature.",
        )
        .condition(cast_a_creature_and_a_noncreature_spell),
    ],
    ..oracle::ESHKI_DRAGONCLAW
};
