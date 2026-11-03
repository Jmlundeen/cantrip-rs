use cantrip_api::*;
use cantrip_mechanics::{effects, tokens, triggers};
use cantrip_oracle as oracle;

/// "When this artifact enters and when you sacrifice it, create a 1/1 white Rabbit creature token and scry 1." (CR 608.2, 111.1, 701.22a).
fn create_1_1_white_rabbit_and_scry(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::RABBIT, 1)?;
    g.end_sentence();
    effects::scry(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS_OR_YOU_SACRIFICE_IT,
            create_1_1_white_rabbit_and_scry,
            "When this artifact enters and when you sacrifice it, create a 1/1 white Rabbit creature token and scry 1.",
        )
        .on_sacrifice(),
        ability(cost("{2}").and(SACRIFICE_THIS).and(TAP), |g, item| effects::gain_life(g, item, 3), "{2}, {T}, Sacrifice this artifact: You gain 3 life."),
    ],
    ..oracle::CARROT_CAKE
};

pub static EXTRA: &[&CardDef] = &[&tokens::RABBIT];
