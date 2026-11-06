use cantrip_api::*;
use cantrip_mechanics::coven;
use cantrip_mechanics::targets::filters::creature;
use cantrip_oracle as oracle;

/// "You may look at the top card of your library any time. You may play lands from the top of your library. Coven - As
/// long as you control three or more creatures with different powers, you may cast creature spells from the top of your
/// library." (CR 604.1).
pub static CARD: CardDef = CardDef {
    play_rules: &[
        PlayRule::LookAtLibraryTop,
        PlayRule::LibraryTopLands,
        PlayRule::While(
            coven::you_control_asked,
            &[PlayRule::LibraryTopSpells(creature, None)],
        ),
    ],
    ..oracle::AUGUR_OF_AUTUMN
};
