use cantrip_api::*;
use cantrip_mechanics::{pump, triggers::ward_discard};
use cantrip_oracle as oracle;

/// "Each non-Dragon creature gets -X/-X until end of turn." The creatures on the battlefield as it resolves that aren't Dragons
/// (CR 608.2c, 611.2c); "Dragon" is a word of the spell's text, read through its words (CR 612). One instruction (CR 608.2c).
fn exude_toxin(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = item.x as i16;
    let dragon = g.words(item.object).subtype(Subtype::Dragon);
    let creatures: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| {
            let chars = g.chars(id);
            chars.is(CardType::Creature) && !chars.has_subtype(dragon)
        })
        .collect();
    instruction(g, |g| {
        for id in creatures {
            pump(g, item.object, id, -x, -x);
        }
    });
    Ok(())
}

/// The Omen (CR 720): cast from the hand as an Omen sorcery for {X}{B}{B}; as it resolves it is shuffled into its owner's library
/// (CR 720.3d, the core's).
pub static EXUDE_TOXIN: CardDef = CardDef { spell: spell(&[], exude_toxin), ..oracle::EXUDE_TOXIN };

pub static CARD: CardDef = CardDef {
    abilities: &[ward_discard("Ward\u{2014}Discard a card. (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player discards a card.)")],
    omen: Some(&EXUDE_TOXIN),
    ..oracle::SCAVENGER_REGENT
};

pub static EXTRA: &[&CardDef] = &[&EXUDE_TOXIN];
