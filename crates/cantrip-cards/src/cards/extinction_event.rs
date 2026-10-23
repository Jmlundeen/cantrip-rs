use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_oracle as oracle;

/// "Choose odd or even. Exile each creature with mana value of the chosen quality. (Zero is even.)" The choice is made as
/// the spell resolves (CR 608.2d), and every player's creatures of that quality are exiled together (CR 406.2).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let me = g.card_id(&CARD);
    let doomed = choose::creatures_of_chosen_quality(g, item, me)?;
    g.exile_all(&doomed);
    Ok(())
}

pub static CARD: CardDef = CardDef { labels: choose::ODD_OR_EVEN, spell: spell(&[], effect), ..oracle::EXTINCTION_EVENT };
