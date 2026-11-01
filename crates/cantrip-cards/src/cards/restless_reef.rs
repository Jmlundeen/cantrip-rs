use cantrip_api::*;
use cantrip_mechanics::{effects, lands, mana, targets, triggers};
use cantrip_oracle as oracle;

/// "Until end of turn, this land becomes a 4/4 blue and black Shark creature with deathtouch. It's still a land."
/// (CR 613.1d, 613.4b, 613.1e): the color and creature type words are read through the land's own text (CR 612).
fn becomes_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let colors = ColorSet(ColorSet::of(Color::Blue).0 | ColorSet::of(Color::Black).0);
    lands::becomes_creature_until_end_of_turn(g, item, (4, 4), colors, Subtype::Shark, Keywords::of(&[Keyword::Deathtouch]));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        mana_ability(TAP, add_one_of(&[Color::Blue, Color::Black]), "{T}: Add {U} or {B}."),
        ability(cost("{2}{U}{B}"), becomes_creature, "{2}{U}{B}: Until end of turn, this land becomes a 4/4 blue and black Shark creature with deathtouch. It's still a land."),
        triggered(triggers::THIS_ATTACKS, effects::target_player_mills::<4>, "Whenever this land attacks, target player mills four cards.")
            .targets(&[targets::PLAYER]),
    ],
    ..oracle::RESTLESS_REEF
};
