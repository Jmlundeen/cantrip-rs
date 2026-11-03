use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// The ability the copy's exception adds (CR 707.9b). Not a card anyone plays: a definition for the exception's
/// ability to be referred to by.
pub static EXCEPTION: CardDef = CardDef {
    name: "Phantasmal Image (copy exception)",
    abilities: &[triggered(triggers::THIS_BECOMES_THE_TARGET_OF_A_SPELL_OR_ABILITY, effects::sacrifice_self, "When this creature becomes the target of a spell or ability, sacrifice it.")],
    ..CardDef::BLANK
};

/// "You may have this creature enter as a copy of any creature on the battlefield, except it's an Illusion in
/// addition to its other types and it has [the exception's ability]." A choice, not a target (CR 608.2d): with one
/// creature out it is taken without asking.
fn enter_as_copy(g: &mut dyn Game, entry: &mut Entering) -> Result<(), Suspend> {
    cantrip_mechanics::copy::enter_as_copy(g, entry, |c| c.is(CardType::Creature), |q, copy| {
        let exception = AbilityRef { card: q.card_id(&EXCEPTION), index: 0 };
        copy.with_ability(exception)
    })?;
    // "It's an Illusion": the creature type is a word of this card's text (CR 612), so it is the type that text names now.
    let illusion = g.words(entry.object).subtype(Subtype::Illusion);
    entry.copy = entry.copy.map(|copy| copy.with_subtype(illusion));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters_as_copy: Some(enter_as_copy),
    ..oracle::PHANTASMAL_IMAGE
};

pub static EXTRA: &[&CardDef] = &[&EXCEPTION];
