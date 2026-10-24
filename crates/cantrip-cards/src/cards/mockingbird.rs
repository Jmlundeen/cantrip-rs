use cantrip_api::*;
use cantrip_mechanics::copy;
use cantrip_oracle as oracle;

const ASK: Question = Question::new(
    "CopyAsEnters",
    "enter as a copy of a creature?",
    "It must have mana value no greater than the mana spent to cast this creature, and it's a Bird in addition to its other types with flying. If you don't, it enters as itself",
    "Choose a creature",
    "Enter as itself",
);

/// "You may have this creature enter as a copy of any creature on the battlefield with mana value less than or equal to the
/// amount of mana spent to cast this creature, except it's a Bird in addition to its other types and it has flying." A
/// replacement effect on its entering (CR 614.1c, 707.2): a choice, not a target (CR 608.2d). All the mana spent to cast it
/// counts, the {X} and the {U} (CR 601.2h), and none was spent if it was not cast. Each creature's mana value is read as it
/// is now (`mana_value`: a token or copy without a cost is 0, an {X} is 0 on the battlefield, CR 202.3). The exceptions are
/// part of the copy's own copiable values (CR 707.9b): the type is read through the words of this card's text (CR 612).
fn enter_as_copy(g: &mut dyn Game, entry: &mut Entering) -> Result<(), Suspend> {
    let entering = entry.object;
    let spent = g.obj(entering).spent.total();
    let candidates: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| id != entering && g.chars(id).is(CardType::Creature) && g.mana_value(id) <= spent)
        .collect();
    copy::enter_as_copy_of_one_of(g, entry, &candidates, |_, copy| copy.with_keyword(Keyword::Flying), &ASK)?;
    let bird = g.words(entering).subtype(Subtype::Bird);
    entry.copy = entry.copy.map(|copy| copy.with_subtype(bird));
    Ok(())
}

pub static CARD: CardDef = CardDef { enters_as_copy: Some(enter_as_copy), ..oracle::MOCKINGBIRD };
