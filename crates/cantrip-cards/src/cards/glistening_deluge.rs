use cantrip_api::*;
use cantrip_mechanics::{effects::pump_all_creatures, pump};
use cantrip_oracle as oracle;

/// "All creatures get -1/-1 until end of turn. Creatures that are green and/or white get an additional -2/-2 until end of
/// turn." Two continuous effects of the one resolution, each fixing its set as it resolves (CR 611.2c), so a creature that
/// enters later, or later becomes green or white, is not affected. The colors are read through the words of the spell's
/// text (CR 612); a creature both green and white is one creature and gets the -2/-2 once. Creatures left with toughness 0
/// or less die as a state-based action (CR 704.5f).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    pump_all_creatures(g, item.object, -1, -1);
    g.end_sentence();
    let words = g.words(item.object);
    let creatures: Vec<ObjectId> = g.state().battlefield().iter().copied().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    let tinted: Vec<ObjectId> = creatures
        .into_iter()
        .filter(|&id| {
            let colors = g.chars(id).colors;
            colors.has(words.color(Color::Green)) || colors.has(words.color(Color::White))
        })
        .collect();
    instruction(g, |g| {
        for id in tinted {
            pump(g, item.object, id, -2, -2);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::GLISTENING_DELUGE };
