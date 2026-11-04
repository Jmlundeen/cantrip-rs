use cantrip_api::*;
use cantrip_mechanics::tokens::{self, LANDER};
use cantrip_mechanics::triggers::{LANDFALL, THIS_ENTERS};
use cantrip_mechanics::{effects, lands_you_control};
use cantrip_oracle as oracle;

/// "When this creature enters, if an opponent controls more lands than you, create a Lander token": the intervening "if"
/// (CR 603.4), checked as the ability triggers and again as it resolves.
fn an_opponent_controls_more(q: &dyn Query, me: &Asker) -> bool {
    let mine = lands_you_control(q, me.controller);
    q.state().opponents(me.controller).any(|opponent| lands_you_control(q, opponent) > mine)
}

fn lander(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_lander(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, lander, "When this creature enters, if an opponent controls more lands than you, create a Lander token.")
            .condition(an_opponent_controls_more),
        triggered(
            LANDFALL,
            effects::this_gets::<1, 0>,
            "Landfall \u{2014} Whenever a land you control enters, this creature gets +1/+0 until end of turn.",
        ),
    ],
    ..oracle::SUNSTAR_EXPANSIONIST
};

pub static EXTRA: &[&CardDef] = &[&LANDER];
