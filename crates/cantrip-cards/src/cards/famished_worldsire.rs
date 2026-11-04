use cantrip_api::*;
use cantrip_mechanics::{devour, effects, triggers};
use cantrip_oracle as oracle;

/// "Look at the top X cards of your library, where X is this creature's power. Put any number of land cards from among them onto the
/// battlefield tapped, then shuffle." X is worked out once, as the ability resolves (ruling 2025-07-25): the power it has then, with
/// the +1/+1 counters it devoured in with and anything else on it.
fn look_for_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = g.chars(item.object).power.unwrap_or_default().max(0) as usize;
    effects::look_at_top_put_any_lands_onto_the_battlefield_tapped_then_shuffle(g, item.controller, item.object, x)
}

pub static CARD: CardDef = CardDef {
    enters: Some(devour::lands::<3>),
    abilities: &[
        triggers::ward::<3>("Ward {3} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {3}.)"),
        triggered(
            triggers::THIS_ENTERS,
            look_for_lands,
            "When this creature enters, look at the top X cards of your library, where X is this creature's power. Put any number of land cards from among them onto the battlefield tapped, then shuffle.",
        ),
    ],
    ..oracle::FAMISHED_WORLDSIRE
};
