use cantrip_api::*;
use cantrip_mechanics::{counter_unless_pays, targets};
use cantrip_oracle as oracle;

/// "Counter target spell unless its controller pays {2} plus an additional {1} for each Faerie you control" (CR 118.12):
/// the Faeries are counted as the spell resolves, among the permanents the Stutter's controller controls (never the spell
/// controller's), through the words of the Stutter's own text (CR 612), and the payment is asked even when nothing more than
/// {2} is owed.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let faerie = g.words(item.object).subtype(Subtype::Faerie);
    let faeries = g.permanents_of(item.controller).into_iter().filter(|&p| g.chars(p).has_subtype(faerie)).count();
    counter_unless_pays(g, item, 0, generic(2 + faeries as u8))?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::SPELL], effect), ..oracle::SPELL_STUTTER };
