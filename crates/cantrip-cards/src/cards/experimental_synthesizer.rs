use cantrip_api::*;
use cantrip_mechanics::library::exile_top_play_this_turn;
use cantrip_mechanics::tokens::{self, SAMURAI};
use cantrip_mechanics::triggers::THIS_ENTERS_OR_LEAVES;
use cantrip_oracle as oracle;

/// "Exile the top card of your library. Until end of turn, you may play that card": a land too, under the ordinary land-play
/// rules (CR 305.1).
fn exile_top_and_play_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_top_play_this_turn(g, item.controller, 1);
    Ok(())
}

/// "Create a 2/2 white Samurai creature token with vigilance."
fn make_samurai(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &SAMURAI, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS_OR_LEAVES,
            exile_top_and_play_it,
            "When this artifact enters or leaves the battlefield, exile the top card of your library. Until end of turn, you may play that card.",
        ),
        ability(
            cost("{2}{R}").and(SACRIFICE_THIS),
            make_samurai,
            "{2}{R}, Sacrifice this artifact: Create a 2/2 white Samurai creature token with vigilance. Activate only as a sorcery.",
        )
        .sorcery_speed(),
    ],
    ..oracle::EXPERIMENTAL_SYNTHESIZER
};

pub static EXTRA: &[&CardDef] = &[&SAMURAI];
