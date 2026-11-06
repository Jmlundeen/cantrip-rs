use cantrip_api::*;
use cantrip_mechanics::{copy, lands};
use cantrip_oracle as oracle;

/// The ability the copy's exception adds (CR 707.9a, 604.3a). Not a card anyone plays: a definition for the exception's
/// characteristic-defining ability to be referred to by. It is part of the copy's copiable values, so a later copy of the
/// creature has it too.
pub static EXCEPTION: CardDef = CardDef {
    name: "Colossal Clone Master (copy exception)",
    text: "This creature's power and toughness are each equal to the number of differently named lands you control.",
    statics: &[characteristic_defining(
        Layer::PtDefine,
        lands::power_and_toughness_are_differently_named_lands_you_control,
        "This creature's power and toughness are each equal to the number of differently named lands you control.",
    )],
    ..CardDef::BLANK
};

/// "You may have this creature enter as a copy of any creature on the battlefield, except it isn't legendary and it has
/// [the exception's ability]." A choice, not a target (CR 614.12, 707.2, 608.2d): after accepting, with one creature out it
/// is taken without asking which, and with none nothing is asked. The exceptions are part of the copy's own copiable values
/// (CR 707.9a-b), so a later copy of this creature is nonlegendary and has the ability too. Declined, it enters as itself, a
/// 0/0 with no exception ability.
fn enter_as_copy(g: &mut dyn Game, entry: &mut Entering) -> Result<(), Suspend> {
    copy::enter_as_copy(g, entry, |c| c.is(CardType::Creature), |q, values| {
        values.nonlegendary().with_text(q.card_id(&EXCEPTION))
    })
}

pub static CARD: CardDef = CardDef { enters_as_copy: Some(enter_as_copy), ..oracle::COLOSSAL_CLONE_MASTER };

pub static EXTRA: &[&CardDef] = &[&EXCEPTION];
