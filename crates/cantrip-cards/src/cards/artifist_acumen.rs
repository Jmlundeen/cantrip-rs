use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Creatures you control gain first strike until end of turn. Draw a card." The creatures are those their controller
/// has as the spell resolves, fixed then (CR 611.2c).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    for id in g.permanents_of(you) {
        if g.chars(id).is(CardType::Creature) {
            g.add_effect(item.object, id, EffectKind::AddKeywords(Keywords::of(&[Keyword::FirstStrike])), Duration::EndOfTurn);
        }
    }
    g.draw(you, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::ARTIFIST_ACUMEN };
