use cantrip_api::*;
use cantrip_mechanics::{attach::attached_to_it, targets};
use cantrip_oracle as oracle;

/// "Enchanted creature's owner shuffles it into their library" (CR 701.24a, 701.24c, 608.2h): the creature this Aura is attached to as the
/// ability resolves, or was as the Aura left the battlefield. It goes to its owner's library, and its owner shuffles, whoever controls it
/// (CR 108.3); a creature that has left the battlefield is nothing to shuffle in, but the owner still shuffles (CR 701.24c).
fn owner_shuffles_enchanted_into_library(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = g.obj(item.object).attached_to {
        g.put_into_library(&[creature], LibraryPlace::ShuffledIn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    abilities: &[ability(
        cost("{5}"),
        owner_shuffles_enchanted_into_library,
        "Waterbend {5}: Enchanted creature's owner shuffles it into their library. (While paying a waterbend cost, you can tap your artifacts and creatures to help. Each one pays for {1}.)",
    )
    .waterbend()],
    untap_bans: &[attached_to_it],
    ..oracle::WATERY_GRASP
};
