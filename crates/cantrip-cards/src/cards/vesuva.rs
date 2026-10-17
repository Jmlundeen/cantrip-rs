use cantrip_api::*;
use cantrip_oracle as oracle;

/// Vesuva's "you may": entering as a copy is entering tapped, so the question says both.
const ENTER_TAPPED_AS_A_COPY: Question = Question::new(
    "CopyAsEnters",
    "enter tapped as a copy of a land?",
    "It enters tapped, as a copy of the land you choose next (the only land, if there is one). If you don't, it enters untapped as itself",
    "Choose a land",
    "Enter as itself",
);

/// "You may have this land enter tapped as a copy of any land on the battlefield." A replacement effect on how it enters
/// (CR 614.1c, 707.2): a choice, not a target (CR 608.2d), among lands already on the battlefield, so one entering at the
/// same moment is no candidate (the ruling). What it copies is the land's copiable values (CR 707.2: the printed card, or
/// what that land is itself copying, never an animation or counters), and that land's own "enters" replacements and
/// "enters" triggers apply as it enters (CR 614.12, 707.9a; the ruling): one that "enters tapped" changes nothing, it
/// enters tapped anyway. If it copies nothing it enters untapped as itself, a land with no abilities (the ruling).
fn enter_tapped_as_copy(g: &mut dyn Game, entry: &mut Entering) -> Result<(), Suspend> {
    cantrip_mechanics::copy::enter_as_copy_asking(g, entry, |c| c.is(CardType::Land), |_, copy| copy, &ENTER_TAPPED_AS_A_COPY)?;
    if entry.copy.is_some() {
        entry.tapped = true;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { enters: Some(enter_tapped_as_copy), ..oracle::VESUVA };
