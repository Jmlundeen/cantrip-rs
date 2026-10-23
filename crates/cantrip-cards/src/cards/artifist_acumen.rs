use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Creatures you control gain first strike until end of turn. Draw a card." The creatures are those their controller
/// has as the spell resolves, fixed then (CR 611.2c).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let creatures: Vec<ObjectId> = g.permanents_of(you).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    instruction(g, |g| {
        for id in creatures {
            effects::gain_until_end_of_turn(g, item.object, id, Keyword::FirstStrike);
        }
    });
    g.end_sentence();
    g.draw(you, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::ARTIFIST_ACUMEN };
