use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, typecycling};
use cantrip_oracle as oracle;

// Trample is read from its JSON.
pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, |g, item| effects::gain_life(g, item, 3), "When this creature enters, you gain 3 life."),
        typecycling::ability(Subtype::Forest, mana("{2}"), "Forestcycling {2} ({2}, Discard this card: Search your library for a Forest card, reveal it, put it into your hand, then shuffle.)"),
    ],
    ..oracle::BALAMB_T_REXAUR
};
