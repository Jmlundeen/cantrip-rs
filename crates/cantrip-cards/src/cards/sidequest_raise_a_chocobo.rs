use cantrip_api::*;
use cantrip_mechanics::triggers::{LANDFALL, THIS_ENTERS, THIS_TRANSFORMS, YOUR_FIRST_MAIN_PHASE};
use cantrip_mechanics::{effects, library, permanents_with_subtype_you_control, pump, tokens};
use cantrip_oracle as oracle;

/// "...if you control four or more Birds" (CR 603.4): an intervening "if", asked as the ability triggers and again as it resolves.
/// Bird is a word of this card's text (CR 612); the Birds are the permanents the layers give that subtype, so a Changeling
/// (CR 702.73a) counts and a token does too.
fn four_or_more_birds(q: &dyn Query, asker: &Asker) -> bool {
    permanents_with_subtype_you_control(q, asker.controller, q.words(asker.source).subtype(Subtype::Bird)) >= 4
}

/// "Birds you control get +1/+0 until end of turn." (CR 611.2, 611.2c): the Birds its controller controls as it resolves, and
/// no Bird that enters later. Bird is a word of this card's text (CR 612); the creatures' types are the layers' answer.
fn birds_get_plus_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let bird = g.words(item.object).subtype(Subtype::Bird);
    let birds: Vec<ObjectId> = g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).has_subtype(bird)).collect();
    instruction(g, |g| {
        for &id in &birds {
            pump(g, item.object, id, 1, 0);
        }
    });
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_TRANSFORMS,
            library::search_for_a_land_onto_the_battlefield_tapped,
            "When this permanent transforms into Black Chocobo, search your library for a land card, put it onto the battlefield tapped, then shuffle.",
        ),
        triggered(LANDFALL, birds_get_plus_one, "Landfall \u{2014} Whenever a land you control enters, Birds you control get +1/+0 until end of turn."),
    ],
    ..oracle::BLACK_CHOCOBO
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            tokens::create_a_landfall_bird,
            "When this enchantment enters, create a 2/2 green Bird creature token with \"Whenever a land you control enters, this token gets +1/+0 until end of turn.\"",
        ),
        triggered(
            YOUR_FIRST_MAIN_PHASE,
            effects::transform_self,
            "At the beginning of your first main phase, if you control four or more Birds, transform this enchantment.",
        )
        .condition(four_or_more_birds),
    ],
    back: Some(&BACK),
    ..oracle::SIDEQUEST_RAISE_A_CHOCOBO
};

pub static EXTRA: &[&CardDef] = &[&tokens::LANDFALL_BIRD];
