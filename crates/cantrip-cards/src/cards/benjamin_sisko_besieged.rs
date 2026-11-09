use cantrip_api::*;
use cantrip_mechanics::targets::filters::creature;
use cantrip_oracle as oracle;

/// "Play with the top card of your library revealed" (CR 401.5, 701.20a).
/// "You may play lands and cast creature spells from the top of your library"
/// (CR 305.1, 601.3): ordinary timing, land limits and all costs still apply.
pub static CARD: CardDef = CardDef {
    play_rules: &[PlayRule::RevealLibraryTop, PlayRule::LibraryTopLands, PlayRule::LibraryTopSpells(creature, None)],
    ..oracle::BENJAMIN_SISKO_BESIEGED
};
