use cantrip_api::*;
use cantrip_oracle as oracle;

/// "You may have this land enter tapped as a copy of any land on the battlefield." A replacement effect on how it enters
/// (CR 614.1c, 707.2): a choice, not a target (CR 608.2d), among lands already on the battlefield, so one entering at the
/// same moment is no candidate (the ruling). What it copies is the land's copiable values (CR 707.2: the printed card, or
/// what that land is itself copying, never an animation or counters), and that land's own "enters" replacements and
/// "enters" triggers apply as it enters (CR 614.12, 707.9a; the ruling): one that "enters tapped" changes nothing, it
/// enters tapped anyway. If it copies nothing it enters untapped as itself, a land with no abilities (the ruling).
fn enter_tapped_as_copy(g: &mut dyn Game, entry: &mut Entering) -> Result<(), Suspend> {
    let entering = entry.object;
    let lands: Vec<ObjectId> = g.state().battlefield().iter().copied().filter(|&id| id != entering && g.chars(id).is(CardType::Land)).collect();
    cantrip_mechanics::copy::enter_tapped_as_copy_of_one_of(g, entry, &lands, |_, copy| copy)
}

pub static CARD: CardDef = CardDef { enters_as_copy: Some(enter_tapped_as_copy), ..oracle::VESUVA };
