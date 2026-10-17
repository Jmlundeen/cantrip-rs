use cantrip_api::*;
use cantrip_mechanics::library::exile_up_to_four_named_from_graveyard_hand_and_library;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Choose a card name. Search target opponent's graveyard, hand, and library for up to four cards with that name and exile
/// them. Then that player shuffles." The shared recipe The Rise of Sozin's chapter II prints too: the name chosen as the
/// spell resolves (CR 608.2c), the three zones searched in the order named, up to four cards in all, exiled at once, then
/// the owner shuffles (CR 701.23, 406.2).
pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::OPPONENT], exile_up_to_four_named_from_graveyard_hand_and_library),
    ..oracle::ANCIENT_VENDETTA
};
