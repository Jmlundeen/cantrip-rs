use cantrip_api::*;
use cantrip_mechanics::attach::equip;
use cantrip_mechanics::replacements::{copies_of_the_attached_permanent, first_tokens_you_would_create_this_turn_while_attached};
use cantrip_oracle as oracle;

const COPY_INSTEAD: Question = Question::new(
    "CreateCopiesOfEquippedCreature",
    "create copies of the equipped creature instead?",
    "The tokens you would create are that many tokens that are copies of the equipped creature",
    "Create copies",
    "Create the tokens",
);

pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana("{2}"), "Equip {2}")],
    replacements: &[replacement(
        ReplacementKind::TokenCreation,
        first_tokens_you_would_create_this_turn_while_attached,
        copies_of_the_attached_permanent,
        "creates that many tokens that are copies of the equipped creature instead",
    )
    .optional(&COPY_INSTEAD)],
    ..oracle::MIRRORMIND_CROWN
};
