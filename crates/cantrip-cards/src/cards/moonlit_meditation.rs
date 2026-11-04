use cantrip_api::*;
use cantrip_mechanics::replacements::{copies_of_the_attached_permanent, first_tokens_you_would_create_this_turn_while_attached};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

const COPY_INSTEAD: Question = Question::new(
    "CreateCopiesOfEnchantedPermanent",
    "create copies of the enchanted permanent instead?",
    "The tokens you would create are that many tokens that are copies of the enchanted permanent",
    "Create copies",
    "Create the tokens",
);

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::ARTIFACT_OR_CREATURE_YOU_CONTROL),
    replacements: &[replacement(
        ReplacementKind::TokenCreation,
        first_tokens_you_would_create_this_turn_while_attached,
        copies_of_the_attached_permanent,
        "creates that many tokens that are copies of the enchanted permanent instead",
    )
    .optional(&COPY_INSTEAD)],
    ..oracle::MOONLIT_MEDITATION
};
