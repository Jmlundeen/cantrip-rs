use cantrip_api::*;
use cantrip_mechanics::battle_cry;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::{THIS_DIES, THIS_ENTERS};
use cantrip_oracle as oracle;

fn make_bat(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::BAT, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        battle_cry::TRIGGER,
        triggered(THIS_ENTERS, make_bat, "When Sanguine Evangelist enters, create a 1/1 black Bat creature token with flying."),
        triggered(THIS_DIES, make_bat, "When Sanguine Evangelist dies, create a 1/1 black Bat creature token with flying."),
    ],
    ..oracle::SANGUINE_EVANGELIST
};

pub static EXTRA: &[&CardDef] = &[&tokens::BAT];
