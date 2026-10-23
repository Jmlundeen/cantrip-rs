use cantrip_api::*;
use cantrip_mechanics::library::{look_at_top, Rest};
use cantrip_mechanics::triggers::{THIS_ENTERS, YOU_CAST_A_SPELL};
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "When this Siege enters, look at the top five cards of your library. You may reveal a permanent card from among them
/// and put it into your hand. Put the rest on the bottom of your library in a random order." (CR 110.4a, 401.4, 701.20a)
fn look(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    look_at_top(g, item.controller, item.object, 5, (0, 1), true, targets::is_permanent_card, Rest::BottomRandom)?;
    Ok(())
}

/// Belligerent Regisaur: "Trample. Whenever you cast a spell, this creature gains indestructible until end of turn."
pub static BACK: CardDef = CardDef {
    abilities: &[triggered(YOU_CAST_A_SPELL, effects::this_gains_indestructible, "Whenever you cast a spell, this creature gains indestructible until end of turn.")],
    ..oracle::BELLIGERENT_REGISAUR
};

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, look, "When this Siege enters, look at the top five cards of your library. You may reveal a permanent card from among them and put it into your hand. Put the rest on the bottom of your library in a random order.")],
    back: Some(&BACK),
    ..oracle::INVASION_OF_IXALAN
};
