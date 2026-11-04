use cantrip_api::*;
use cantrip_mechanics::{effects, library, targets, tiered};
use cantrip_oracle as oracle;

/// "Target creature's owner shuffles it into their library." (CR 701.24a)
fn shuffle_it_in(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        g.put_into_library(&[permanent], LibraryPlace::ShuffledIn);
    }
    Ok(())
}

const MODES: &[ModeDef] = &[
    mode(effects::return_first_target_to_hand, "Blizzard — {0} — Return target creature to its owner's hand.")
        .targets(&[targets::CREATURE])
        .cost(mana("{0}")),
    mode(
        library::owner_puts_first_target_on_top_or_bottom,
        "Blizzara — {2} — Target creature's owner puts it on their choice of the top or bottom of their library.",
    )
    .targets(&[targets::CREATURE])
    .cost(mana("{2}")),
    mode(shuffle_it_in, "Blizzaga — {5}{U} — Target creature's owner shuffles it into their library.")
        .targets(&[targets::CREATURE])
        .cost(mana("{5}{U}")),
];

static MODAL: ModalDef = tiered::modal(MODES);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::ICE_MAGIC };
